use std::{
    io::{Cursor, Read},
    sync::Arc,
};

use anyhow::Error;
use audiotags::MimeType;
use bytes::Bytes;
use iced::widget::image::Handle;
use image::{
    DynamicImage, GenericImageView, ImageFormat, ImageReader, Rgb,
    codecs::jpeg::JpegEncoder,
    imageops::FilterType::{Gaussian, Nearest, Triangle},
};
use image_compare::{Algorithm::MSSIMSimple, gray_similarity_structure};
use rand::RngCore;
use tokio::{sync::Semaphore, task::yield_now};

use crate::{
    ImgHandle,
    api::queue::ReturnSongImg,
    app::{
        song::GroupSize,
        song_img::{ImgFormat, LazyImage, SongImg},
    },
    parser::file_parser::TagData,
};

#[derive(Clone, Copy, Debug)]
pub struct ImageSettings {
    pub downscale: u32,
    pub square: bool,
    pub jpg: bool,
    pub hash: u32,
}

impl Default for ImageSettings {
    fn default() -> Self {
        Self {
            downscale: 1200,
            jpg: true,
            square: false,
            hash: rand::rng().next_u32(),
        }
    }
}

const PREVIEW_DIM: u32 = 200;
const COMPARE_DIM: u32 = 100;
pub async fn decode_and_sample(
    mut new: ReturnSongImg,
    sem: Arc<Semaphore>,
    set: ImageSettings,
) -> Result<ReturnSongImg, Error> {
    let permit = sem.acquire().await.unwrap();

    let bytes = new.img.raw();
    let mut decoded =
        ImageReader::with_format(Cursor::new(bytes), new.img.orig_format.imageio()).decode()?;
    new.img.image = LazyImage::Decoded(decoded.clone());

    yield_now().await;

    let (w, h) = decoded.dimensions();
    new.img.orig_res = (w, h);

    if set.square && w > h {
        decoded = decoded.crop_imm(w / 2 - h / 2, 0, h, h);
    }

    let dyn_img = decoded.thumbnail(PREVIEW_DIM, PREVIEW_DIM);
    yield_now().await;
    let prev_dim = dyn_img.dimensions();
    let dyn_clone = dyn_img.clone();
    let dyn_img = dyn_img.thumbnail_exact(COMPARE_DIM, COMPARE_DIM);
    yield_now().await;

    let prev = dyn_clone.into_rgba8().into_vec();
    let prev = Bytes::from_owner(prev);
    new.img.preview = Some(Handle::from_rgba(prev_dim.0, prev_dim.1, prev));

    let samp = dyn_img.clone().into_luma8();
    new.img.sample = Some(samp);

    drop(permit);
    Ok(new)
}
pub fn original_image_preview(tag: &TagData) -> Option<ImgHandle> {
    let img = tag.file.album_cover()?;
    let bytes = Bytes::from_owner(img.data.to_owned());

    let preprocessed = ImageReader::with_format(Cursor::new(bytes), from_mime(img.mime_type))
        .decode()
        .ok()?;
    let preprocessed = preprocessed.resize_to_fill(PREVIEW_DIM, PREVIEW_DIM, Nearest);
    let (w, h) = preprocessed.dimensions();
    let rgb = preprocessed.to_rgba8();
    let bytes = Bytes::from_owner(rgb.into_raw());
    Some(Handle::from_rgba(w, h, bytes))
}
const THRESHOLD: f64 = 0.3;
const SORT_LIMIT: usize = 15;
pub fn push_and_group(
    groups: &mut Vec<GroupSize>,
    all: &mut Vec<SongImg>,
    mut new: SongImg,
) -> Result<(), Error> {
    if new.sample.is_none() {
        all.push(new);
        return Ok(());
    }
    let b = new.sample.unwrap();

    // let (b_w, b_h) = b.dimensions();
    // iterate over first element of each group
    let mut end_of_groups = 0;
    let mut group = 0;
    let mut i = 0;
    while i < all.len() {
        // limit sorting time, more groups reach limit faster
        if i > SORT_LIMIT {
            new.sample = Some(b);
            all.push(new);
            return Ok(());
        }
        if let Some(a) = all[i].sample.as_ref() {
            let score = gray_similarity_structure(&MSSIMSimple, a, &b)?.score;

            if score > THRESHOLD {
                dbg!(score);
                break;
            }
        }

        if group < groups.len() {
            i += groups[group] as usize;
            end_of_groups += groups[group] as usize;
            group += 1;
        } else {
            i += 1;
        }
    }
    new.sample = Some(b);

    // not found / empty
    if i == all.len() {
        all.push(new);
        return Ok(());
    }

    let mut group_first_element = i;
    // within existing group
    if group < groups.len() {
        // group is index groug to insert into

        // rearrange groups until sorted by group size
        while group > 0 && groups[group] + 1 > groups[group - 1] {
            // swap each element
            for group_i in 0..groups[group] as usize {
                all.swap(
                    group_first_element + group_i,
                    group_first_element + group_i - groups[group] as usize,
                );
            }
            // update to point to swapped group
            group_first_element -= groups[group] as usize;
            group -= 1;
        }
        let mut group_i = 0;
        while group_i < groups[group] as usize
            && all[group_first_element + group_i].decoded().dimensions().1
                > new.decoded().dimensions().1
        {
            group_i += 1;
        }
        groups[group] += 1;
        all.insert(group_first_element + group_i, new);
        return Ok(());
    }
    // form new group
    all.swap(end_of_groups, group_first_element);
    if all[end_of_groups].decoded().dimensions().1 > new.decoded().dimensions().1 {
        all.insert(end_of_groups + 1, new);
    } else {
        all.insert(end_of_groups, new);
    }
    groups.push(2);

    Ok(())
}

pub fn final_img_preview(img: &mut SongImg, set: &ImageSettings) -> ImgHandle {
    let scaled = apply_settings(img, set);
    let (w, h) = scaled.dimensions();

    let scaled = scaled.to_rgba8();
    ImgHandle::from_rgba(w, h, Bytes::from_owner(scaled.into_raw()))
}
pub fn final_img(img: &mut SongImg, set: &ImageSettings) -> (Bytes, ImgFormat, Handle) {
    let scaled = apply_settings(img, set);

    let (w, h) = scaled.dimensions();

    let preview = scaled.to_rgba8();
    let handle = ImgHandle::from_rgba(w, h, Bytes::from_owner(preview.into_raw()));

    let mut new_img = Vec::<u8>::new();
    let mut buf = Cursor::new(&mut new_img);
    let format = if set.jpg {
        ImgFormat::Jpg
    } else {
        img.orig_format.clone()
    };
    scaled.write_to(&mut buf, format.imageio()).unwrap();
    (Bytes::from_owner(new_img), format, handle)
}
fn apply_settings(img: &mut SongImg, set: &ImageSettings) -> DynamicImage {
    let raw = img.decoded();

    let (w, h) = raw.dimensions();
    let cropped = if set.square && w > h {
        raw.crop_imm(w / 2 - h / 2, 0, h, h)
    } else {
        raw
    };
    let (_, h) = cropped.dimensions();

    if set.downscale < h {
        cropped.resize(9999, set.downscale, Triangle)
    } else {
        cropped
    }
}

fn from_mime(mime: MimeType) -> ImageFormat {
    match mime {
        MimeType::Gif => ImageFormat::Gif,
        MimeType::Png => ImageFormat::Png,
        MimeType::Tiff => ImageFormat::Tiff,
        MimeType::Bmp => ImageFormat::Bmp,
        MimeType::Jpeg => ImageFormat::Jpeg,
    }
}
