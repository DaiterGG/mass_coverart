use audiotags::MimeType;
use bytes::Bytes;
use image::{DynamicImage, ImageBuffer, ImageFormat, Luma};
use log::{info, warn};
use rand::{RngCore, rng};

use crate::{ImgHandle, api::queue::Source};

use std::{io::Cursor, sync::Arc};

use anyhow::{Error, bail};
use iced::widget::image::Handle;
use image::{
    GenericImageView, ImageReader,
    imageops::FilterType::{Nearest, Triangle},
};
use image_compare::{Algorithm::MSSIMSimple, gray_similarity_structure};
use tokio::{sync::Semaphore, task::yield_now};

use crate::{app::song::GroupSize, parser::file_parser::TagData};

const THRESHOLD: f64 = 0.3;
const SORT_LIMIT: usize = 15;
const PREVIEW_DIM: u32 = 200;
const COMPARE_DIM: u32 = 200;

#[derive(Clone, Copy, Debug)]
pub struct ImageSettings {
    pub downscale: u32,
    pub square: bool,
    pub jpg: bool,
}

impl Default for ImageSettings {
    fn default() -> Self {
        Self {
            downscale: 1200,
            jpg: true,
            square: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ImgFormat {
    Jpg,
    Png,
}
impl ImgFormat {
    pub fn imageio(&self) -> ImageFormat {
        match self {
            Self::Png => ImageFormat::Png,
            Self::Jpg => ImageFormat::Jpeg,
        }
    }
    pub fn audiotags(&self) -> MimeType {
        match self {
            Self::Png => MimeType::Png,
            Self::Jpg => MimeType::Jpeg,
        }
    }
    pub fn from_url(url: &str) -> Self {
        // Remove query parameters and fragments
        let clean_url = url.split(['?', '#']).next().unwrap_or(url);

        // Extract file extension
        let extension = clean_url.rsplit('.').next().unwrap_or("").to_lowercase();

        match extension.as_str() {
            "jpg" | "jpeg" => ImgFormat::Jpg,
            "png" => ImgFormat::Png,
            _ => {
                // Try to detect from the entire URL as fallback
                let url_lower = url.to_lowercase();
                if url_lower.contains(".png") {
                    ImgFormat::Png
                } else if url_lower.contains(".jpg") || url_lower.contains(".jpeg") {
                    ImgFormat::Jpg
                } else {
                    // Default fallback
                    warn!("format was not parsed from url {}", url);
                    ImgFormat::Jpg
                }
            }
        }
    }
}
type SortSample = ImageBuffer<Luma<u8>, Vec<u8>>;
#[derive(Clone, Debug)]
pub enum ImageProgress {
    RawPreview(Vec<String>, Bytes),
    Raw(Bytes),
    Preview(Vec<String>),
    Decoded(DynamicImage),
}
impl ImageProgress {
    pub fn dbg(&self) -> String {
        match self {
            Self::RawPreview(_, _) => "RawP".to_string(),
            Self::Raw(_) => "Raw".to_string(),
            Self::Preview(_) => "Prev".to_string(),
            Self::Decoded(_) => "Dec".to_string(),
        }
    }
}

pub type ImgId = usize;
pub type ImgHash = u64;
#[derive(Clone, Debug)]
/// * `orig_format`: format of the full image, preview image format will be guessed
pub struct SongImg {
    pub hash: ImgHash,
    pub orig_format: ImgFormat,
    pub src: Source,
    pub image: ImageProgress,
    pub orig_res: Option<(u32, u32)>,
    pub preview: Option<ImgHandle>,
    pub sample: Option<SortSample>,
    pub feedback: String,
}
impl SongImg {
    pub fn new(format: ImgFormat, image: ImageProgress, src: Source, feedback: String) -> Self {
        Self {
            image,
            orig_format: format,
            hash: rng().next_u64(),
            src,
            orig_res: None,
            preview: None,
            sample: None,
            feedback,
        }
    }
    pub fn decoded(&self) -> DynamicImage {
        match &self.image {
            ImageProgress::Decoded(d) => d.clone(),
            _ => panic!(""),
        }
    }

    pub async fn decode_and_sample(mut self, sem: Arc<Semaphore>) -> Result<SongImg, Error> {
        let permit = sem.acquire().await.unwrap();

        let (urls, bytes) = match &mut self.image {
            ImageProgress::Raw(b) => (None, b),
            ImageProgress::RawPreview(url, b) => (Some(url), b),
            _ => panic!("not raw"),
        };

        // small preview img can be in different format for ex. png and 250jpg
        let res = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()?
            .decode();
        if let Err(e) = res {
            bail!(
                "preview img was not decoded: {e}, format: {:?}, urls: {:?}, feedback: {} ",
                self.orig_format,
                urls,
                self.feedback
            );
        }

        let decoded = res.unwrap();
        let (w, h) = decoded.dimensions();
        if let Some(urls) = urls {
            self.image = ImageProgress::Preview(urls.to_vec());
        } else {
            self.orig_res = Some((w, h));
            self.image = ImageProgress::Decoded(decoded.clone());
        }

        yield_now().await;

        let dyn_img = decoded.thumbnail(PREVIEW_DIM, PREVIEW_DIM);
        yield_now().await;
        let (w, h) = dyn_img.dimensions();
        let dyn_clone = dyn_img.clone();

        let dyn_img = dyn_img.crop_imm(w / 2 - h / 2, 0, h, h);

        let dyn_img = dyn_img.thumbnail_exact(COMPARE_DIM, COMPARE_DIM);
        yield_now().await;

        let prev = dyn_clone.into_rgba8().into_vec();
        let prev = Bytes::from_owner(prev);
        self.preview = Some(Handle::from_rgba(w, h, prev));

        let samp = dyn_img.clone().into_luma8();
        self.sample = Some(samp);

        drop(permit);
        Ok(self)
    }
    pub fn original_image_preview(tag: &TagData) -> Option<ImgHandle> {
        let img = tag.file.album_cover()?;
        let bytes = Bytes::from_owner(img.data.to_owned());

        let preprocessed = ImageReader::with_format(Cursor::new(bytes), from_mime(img.mime_type))
            .decode()
            .ok()?;
        let preprocessed = preprocessed.thumbnail(PREVIEW_DIM * 2, PREVIEW_DIM);
        let (w, h) = preprocessed.dimensions();
        let rgb = preprocessed.to_rgba8();
        let bytes = Bytes::from_owner(rgb.into_raw());
        Some(Handle::from_rgba(w, h, bytes))
    }
    pub fn push_and_group(
        mut self,
        groups: &mut Vec<GroupSize>,
        all: &mut Vec<SongImg>,
    ) -> Result<(), Error> {
        if self.sample.is_none() {
            all.push(self);
            return Ok(());
        }
        let b = self.sample.unwrap();

        // iterate over first element of each group
        let mut end_of_groups = 0;
        let mut group = 0;
        let mut i = 0;
        while i < all.len() {
            // limit sorting time, more groups - reach limit faster
            if i > SORT_LIMIT {
                self.sample = Some(b);
                all.push(self);
                return Ok(());
            }
            if let Some(a) = all[i].sample.as_ref() {
                let score = gray_similarity_structure(&MSSIMSimple, a, &b)?.score;

                if score > THRESHOLD {
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
        self.sample = Some(b);

        // not found / empty
        if i == all.len() {
            all.push(self);
            return Ok(());
        }

        let mut group_first_element = i;
        // within existing group
        if group < groups.len() {
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
                && all[group_first_element + group_i]
                    .orig_res
                    .unwrap_or_default()
                    .1
                    > self.orig_res.unwrap_or_default().1
            {
                group_i += 1;
            }
            groups[group] += 1;
            all.insert(group_first_element + group_i, self);
            return Ok(());
        }
        // form new group
        all.swap(end_of_groups, group_first_element);
        if all[end_of_groups].orig_res.unwrap_or_default().1 > self.orig_res.unwrap_or_default().1 {
            all.insert(end_of_groups + 1, self);
        } else {
            all.insert(end_of_groups, self);
        }
        groups.push(2);

        Ok(())
    }

    pub fn preview_to_decoded(&mut self, bytes: Bytes, format: ImgFormat) -> Result<(), Error> {
        self.orig_format = format;

        let preprocessed =
            ImageReader::with_format(Cursor::new(bytes), self.orig_format.imageio()).decode()?;
        let (w, h) = preprocessed.dimensions();
        self.orig_res = Some((w, h));

        info!("img decoded {}", self.feedback);
        self.image = ImageProgress::Decoded(preprocessed);
        Ok(())
    }
    /// self.image has to be decoded
    pub fn final_img_preview(&mut self, set: ImageSettings) -> ImgHandle {
        let scaled = self.apply_settings(&set);
        let (w, h) = scaled.dimensions();

        let scaled = scaled.to_rgba8();
        ImgHandle::from_rgba(w, h, Bytes::from_owner(scaled.into_raw()))
    }
    /// self.image has to be decoded
    pub fn final_img(&mut self, set: &ImageSettings) -> (Bytes, ImgFormat, Handle) {
        let scaled = self.apply_settings(set);

        let (w, h) = scaled.dimensions();

        let preview = scaled.to_rgba8();
        let handle = ImgHandle::from_rgba(w, h, Bytes::from_owner(preview.into_raw()));

        let mut new_img = Vec::<u8>::new();
        let mut buf = Cursor::new(&mut new_img);
        let format = if set.jpg {
            ImgFormat::Jpg
        } else {
            self.orig_format
        };
        scaled.write_to(&mut buf, format.imageio()).unwrap();
        (Bytes::from_owner(new_img), format, handle)
    }
    fn apply_settings(&mut self, set: &ImageSettings) -> DynamicImage {
        let raw = self.decoded();

        let (w, h) = raw.dimensions();
        let cropped = if set.square && w > h {
            raw.crop_imm(w / 2 - h / 2, 0, h, h)
        } else {
            raw
        };
        let (w, h) = cropped.dimensions();

        if set.downscale < h {
            cropped.resize(9999, set.downscale, Triangle)
        } else {
            cropped
        }
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
