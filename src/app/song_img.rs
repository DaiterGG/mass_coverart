use audiotags::MimeType;
use bytes::Bytes;
use image::{DynamicImage, ImageBuffer, ImageFormat, Luma};
use rand::{RngCore, rng};

use crate::{
    ImgHandle,
    api::queue::Source,
    parser::image_parser::{ImageSettings, final_img, final_img_preview},
};

#[derive(Clone, Debug)]
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
}
type Resolution = (u32, u32);
type SortSample = ImageBuffer<Luma<u8>, Vec<u8>>;
#[derive(Clone, Debug)]
pub enum LazyImage {
    Raw(Bytes),
    Decoded(DynamicImage),
}

pub type ImgId = usize;
pub type ImgHash = u64;
#[derive(Clone, Debug)]
pub struct SongImg {
    pub hash: ImgHash,
    pub orig_format: ImgFormat,
    pub orig_res: (u32, u32),
    pub src: Source,
    pub image: LazyImage,
    pub preview: Option<ImgHandle>,
    pub sample: Option<SortSample>,
}
impl SongImg {
    pub fn new(bytes: Bytes, format: ImgFormat, src: Source) -> Self {
        Self {
            image: LazyImage::Raw(bytes),
            orig_format: format,
            hash: rng().next_u64(),
            orig_res: (0, 0),
            src,
            preview: None,
            sample: None,
        }
    }
    pub fn raw(&self) -> Bytes {
        match &self.image {
            LazyImage::Raw(b) => b.clone(),
            _ => panic!("not raw"),
        }
    }
    pub fn decoded(&self) -> DynamicImage {
        match &self.image {
            LazyImage::Decoded(b) => b.clone(),
            _ => panic!("not decoded"),
        }
    }
    pub fn get_final_preview(&mut self, set: &ImageSettings) -> ImgHandle {
        final_img_preview(self, set)
    }
}
