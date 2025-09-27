use std::io::Cursor;

use bytes::Bytes;
use image::{GenericImageView, ImageBuffer, ImageFormat, ImageReader, Luma, Rgb, Rgba};
use rand::{RngCore, rng};

use crate::{ImgHandle, api::queue::Source};

#[derive(Clone, Debug)]
pub enum ImgFormat {
    Jpg,
    Png,
}
impl ImgFormat {
    pub fn convert(&self) -> ImageFormat {
        match self {
            Self::Png => ImageFormat::Png,
            Self::Jpg => ImageFormat::Jpeg,
        }
    }
}
pub type ImgHash = u64;
#[derive(Clone, Debug)]
pub struct SongImg {
    pub hash: ImgHash,
    pub handle: ImgHandle,
    pub preview: Option<ImgHandle>,
    pub format: ImgFormat,
    pub sort_sample: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,
    pub src: Source,
    resolution: Option<(u32, u32)>,
}
impl SongImg {
    pub fn new(handle: ImgHandle, format: ImgFormat, src: Source) -> Self {
        Self {
            handle,
            format,
            preview: None,
            resolution: None,
            hash: rng().next_u64(),
            sort_sample: None,
            src,
        }
    }
    pub fn bytes(&self) -> Bytes {
        match self.handle.clone() {
            ImgHandle::Bytes(id, b) => {
                return b;
            }
            _ => {
                panic!("only bytes supported");
            }
        }
    }
    pub fn set_resolution(&mut self, res: (u32, u32)) {
        self.resolution = Some(res);
    }
    // second chance at avoiding invariant
    pub fn resolution(&self) -> (u32, u32) {
        if let Some(res) = self.resolution {
            return res;
        }
        dbg!("invariant backup");
        ImageReader::with_format(Cursor::new(self.bytes().clone()), self.format.convert())
            .decode()
            .unwrap()
            .dimensions()
    }
}
