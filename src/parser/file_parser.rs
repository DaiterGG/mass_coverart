use anyhow::{Error, bail};
use iced::Length::Fill;
use std::{
    fs::{read, read_dir},
    path::{Path, PathBuf},
};

use audiotags::{AudioTag, Tag};
use thiserror::Error;

#[derive(Debug)]
pub enum RegexType {
    Album,
    Title,
    Artist,
    None,
}

impl RegexType {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Album => "album",
            Self::Title => "title",
            Self::Artist => "artist",
            Self::None => "skip",
        }
    }
    pub fn next(&self) -> Self {
        match self {
            Self::Album => Self::Title,
            Self::Title => Self::Artist,
            Self::Artist => Self::None,
            Self::None => Self::Album,
        }
    }
    fn reg_to_settings(&self, entry: &str, data: &mut TagData) {
        match self {
            Self::Album => {
                data.album.get_or_insert(entry.to_string());
            }
            Self::Title => {
                data.title.get_or_insert(entry.to_string());
            }
            Self::Artist => {
                data.artist.get_or_insert(entry.to_string());
            }
            Self::None => {}
        };
    }
}
pub enum EndFormat {
    Jpeg,
    Png,
}

pub struct ImageSettings {
    pub convert_to: EndFormat,
    pub downscale: (i32, i32),
}

impl Default for ImageSettings {
    fn default() -> Self {
        Self {
            downscale: (1200, 1200),
            convert_to: EndFormat::Jpeg,
        }
    }
}
pub struct ParseSettings {
    pub recursive: bool,
    pub parse_file_name: bool,
    pub reg_keys: Vec<RegexType>,
    pub reg_separators: Vec<String>,
}

impl Default for ParseSettings {
    fn default() -> Self {
        Self {
            recursive: true,
            parse_file_name: false,
            reg_keys: vec![RegexType::Artist, RegexType::Title],
            reg_separators: vec![" - ".to_string()],
        }
    }
}
pub type FileData = Box<dyn AudioTag + Send + Sync + 'static>;
pub struct TagData {
    pub path: PathBuf,
    file: FileData,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
}
impl TagData {
    fn new(path: PathBuf, file: FileData) -> Self {
        Self {
            path,
            file,
            artist: None,
            title: None,
            album: None,
        }
    }
    fn print(&self) {
        println!("{:?}", self.path);
        println!("{:?}", self.artist);
        println!("{:?}", self.title);
        println!("{:?}", self.album);
    }
}
pub struct FileParser;
impl FileParser {
    pub fn get_tags_data(path: PathBuf, set: &ParseSettings) -> Result<Vec<TagData>, Error> {
        let mut pair = Self::parse_path(path, set.recursive)?;
        let mut log = pair.1.rsplitn(30, '\n').collect::<Vec<&str>>();
        log.pop();
        println!("{}", log.join("\n"));
        for file in &mut pair.0 {
            Self::parse_tags(file, set);
            file.print();
        }
        Ok(pair.0)
    }
    pub fn parse_tags(data: &mut TagData, set: &ParseSettings) {
        data.artist = data.file.artist().map(|s| s.to_string());
        data.title = data.file.title().map(|s| s.to_string());
        data.album = data.file.album_title().map(|s| s.to_string());
        if !set.parse_file_name {
            return;
        }
        let mut remaider = data.path.file_stem().unwrap().to_string_lossy().to_string();
        for i in 0..set.reg_separators.len() {
            let sep = &set.reg_separators[i];
            if let Some((entry, rest)) = remaider.split_once(sep) {
                set.reg_keys[i].reg_to_settings(entry, data);
                remaider = rest.to_string();
            }
        }
        set.reg_keys
            .last()
            .unwrap()
            .reg_to_settings(&remaider, data);
    }
    pub fn parse_path(path: PathBuf, rec: bool) -> Result<(Vec<TagData>, String), Error> {
        let mut log = "".to_string();
        let mut all_files: Vec<TagData> = Vec::new();
        let p: &Path = path.as_ref();
        if p.is_file() {
            let res = Self::parse_file(path);
            match res {
                Ok(file) => all_files.push(file),
                // skip errors on individual files
                Err(e) => {
                    log.push_str(&e.to_string());
                    log.push('\n');
                }
            }
        } else if p.is_dir() {
            let all = read_dir(path)?;
            for item in all {
                let item = item?.path();
                if !rec && item.is_dir() {
                    continue;
                }
                let res = Self::parse_path(item, true);
                match res {
                    Ok((mut files, l)) => {
                        all_files.append(&mut files);
                        log.push_str(&l);
                    }
                    // skip errors in subdir
                    Err(e) => log.push_str(&e.to_string()),
                }
            }
        } else if p.is_symlink() {
            log.push_str("Root folder cannot be a Symlink\n");
            bail!(log);
        } else {
            log.push_str("Unknown error\n");
            bail!(log);
        }
        if all_files.is_empty() {
            log.push_str("Music files was not found\n");
            bail!(log);
        }
        Ok((all_files, log))
    }
    pub fn parse_file(path: PathBuf) -> Result<TagData, Error> {
        let file = Tag::new().read_from_path(&path)?;
        Ok(TagData::new(path, file))
    }
    pub fn is_rtl(s: &str) -> bool {
        let cs = s.chars();
        for c in cs {
            match c {
                '\u{590}'..='\u{8FF}' | '\u{10800}'..='\u{11000}' => return true,
                _ => (),
            };
        }
        return false;
    }
}
