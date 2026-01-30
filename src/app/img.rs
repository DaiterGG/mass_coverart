use audiotags::{MimeType, Picture};
use bytes::Bytes;
use image::{DynamicImage, ImageBuffer, ImageFormat, Luma};
use log::{info, warn};

use crate::{ImgHandle, api::queue::Source, app::img_group::ImgGroups};

use std::{io::Cursor, sync::Arc};

use anyhow::{Error, bail};
use iced::widget::image::Handle;
use image::{GenericImageView, ImageReader, imageops::FilterType::Triangle};
use image_compare::{Algorithm::MSSIMSimple, gray_similarity_structure};
use tokio::{sync::Semaphore, task::yield_now};

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
    Jpeg,
    Png,
}
impl ImgFormat {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }
    pub fn imageio(&self) -> ImageFormat {
        match self {
            Self::Png => ImageFormat::Png,
            Self::Jpeg => ImageFormat::Jpeg,
        }
    }
    pub fn audiotags(&self) -> MimeType {
        match self {
            Self::Png => MimeType::Png,
            Self::Jpeg => MimeType::Jpeg,
        }
    }

    pub fn from_imageio(format: ImageFormat) -> Self {
        match format {
            ImageFormat::Png => Self::Png,
            ImageFormat::Jpeg => Self::Jpeg,
            _ => Self::Jpeg,
        }
    }
    pub fn from_url(url: &str) -> Self {
        let clean_url = url.split(['?', '#']).next().unwrap_or(url);

        let extension = clean_url.rsplit('.').next().unwrap_or("").to_lowercase();

        match extension.as_str() {
            "jpg" | "jpeg" => ImgFormat::Jpeg,
            "png" => ImgFormat::Png,
            _ => {
                // Try to detect from the entire URL as fallback
                let url_lower = url.to_lowercase();
                if url_lower.contains(".png") {
                    ImgFormat::Png
                } else if url_lower.contains(".jpg") || url_lower.contains(".jpeg") {
                    ImgFormat::Jpeg
                } else {
                    // Default fallback
                    warn!("format was not parsed from url {}", url);
                    ImgFormat::Jpeg
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
#[derive(Clone, Debug)]
/// * `orig_format`: format of the full image, preview image format will be guessed
pub struct SongImg {
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

        self.sample = match self.src {
            Source::LocalFile => {
                info!("local file, skipping sample gen");
                None
            }
            _ => Some(dyn_img.clone().into_luma8()),
        };

        drop(permit);
        Ok(self)
    }
    pub async fn original_image_preview(img: Vec<u8>, mime_type: MimeType) -> Option<ImgHandle> {
        let img = Picture {
            data: &img,
            mime_type,
        };
        let bytes = Bytes::from_owner(img.data.to_owned());

        let preprocessed = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .ok()?
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
        groups: &mut ImgGroups,
        all: &mut Vec<SongImg>,
    ) -> Result<(), Error> {
        if self.sample.is_none() {
            groups.add_new(all.len(), self.src.get_weight());
            all.push(self);
            return Ok(());
        }
        let b = self.sample.unwrap();

        for group_i in 0..groups.len() {
            if group_i > SORT_LIMIT {
                break;
            }
            if let Some(a) = all[groups.first_in_group(group_i)].sample.as_ref() {
                let score = gray_similarity_structure(&MSSIMSimple, a, &b)?.score;

                info!("threshold: {}", score);
                if score > THRESHOLD {
                    self.sample = Some(b);
                    groups.add_to_group(group_i, &self, all.len(), all);
                    all.push(self);
                    return Ok(());
                }
            }
        }
        groups.add_new(all.len(), self.src.get_weight());
        self.sample = Some(b);
        all.push(self);
        Ok(())
    }

    pub fn preview_to_decoded(&mut self, bytes: Bytes, format: ImgFormat) -> Result<(), Error> {
        self.orig_format = format;

        let guessed = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;

        // in case url extension lied
        self.orig_format = ImgFormat::from_imageio(guessed.format().unwrap());

        let preprocessed = guessed.decode()?;
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

        let preview = scaled.thumbnail(PREVIEW_DIM * 2, PREVIEW_DIM);
        let (w, h) = preview.dimensions();
        let preview = preview.to_rgba8();
        let handle = ImgHandle::from_rgba(w, h, Bytes::from_owner(preview.into_raw()));

        let mut new_img = Vec::<u8>::new();
        let mut buf = Cursor::new(&mut new_img);
        let format = if set.jpg {
            ImgFormat::Jpeg
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
        let (_, h) = cropped.dimensions();

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
