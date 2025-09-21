use anyhow::{Error, bail};
use iced::Length::Fill;
use std::{
    fs::{read, read_dir},
    path::{Path, PathBuf},
};

use audiotags::{AudioTag, Tag};
use thiserror::Error;

struct FileTags {
    path: PathBuf,
    tag: Box<dyn AudioTag + Send + Sync + 'static>,
}
pub struct TagData {
    //
}
pub struct FileParser;
impl FileParser {
    pub fn get_tags_data(path: PathBuf) -> Result<Vec<TagData>, Error> {
        let filetags = Self::parse_path(path)?;
        Ok(vec![])
    }
    pub fn parse_path(path: PathBuf) -> Result<Vec<FileTags>, Error> {
        let mut all_files: Vec<FileTags> = Vec::new();
        let p: &Path = path.as_ref();
        if p.is_file() {
            let res = Self::parse_file(path);
            match res {
                Ok(file) => all_files.push(file),
                // skip errors on individual files
                Err(e) => println!("Skipped: {e:?}"),
            }
        } else if p.is_dir() {
            let all = read_dir(path)?;
            for item in all {
                let item = item?.path();
                let res = Self::parse_path(item);
                match res {
                    Ok(mut files) => all_files.append(&mut files),
                    // skip errors in subdir
                    Err(e) => println!("Skipped: {e:?}"),
                }
            }
        } else if p.is_symlink() {
            bail!("Root folder cannot be a Symlink");
        } else {
            bail!("Unknown error");
        }
        if all_files.is_empty() {
            bail!("Music files was not found");
        }
        Ok(all_files)
    }
    pub fn parse_file(path: PathBuf) -> Result<FileTags, Error> {
        let tag = Tag::new().read_from_path(&path)?;
        Ok(FileTags { tag, path })
    }
}
// let pic_data: Vec<u8> = read(path).unwrap();
