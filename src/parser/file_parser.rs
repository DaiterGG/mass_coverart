use anyhow::{Error, bail};
use iced::Length::Fill;
use rfd::FileHandle;
use std::{
    fmt::Debug,
    fs::{read, read_dir},
    path::{Path, PathBuf},
};

use audiotags::{AudioTag, Picture, Tag};
use thiserror::Error;

use crate::app::{
    iced_app::CoverUI,
    song::{Song, SongId},
};

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
    pub file: FileData,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
}
impl Debug for TagData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl Clone for TagData {
    fn clone(&self) -> Self {
        panic!("should not be cloned")
    }
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
}
pub async fn get_tags_data(
    path_vec: Vec<FileHandle>,
    set: ParseSettings,
) -> Result<Vec<TagData>, Error> {
    let mut ret = Vec::new();
    for path in path_vec {
        let mut tags = parse_path(path.into(), set.recursive)?;
        for file in &mut tags {
            parse_tags(file, &set);
        }
        ret.append(&mut tags);
    }
    Ok(ret)
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
pub fn parse_path(path: PathBuf, rec: bool) -> Result<Vec<TagData>, Error> {
    let mut all_files: Vec<TagData> = Vec::new();
    let p: &Path = path.as_ref();
    if p.is_file() {
        let res = parse_file(path);
        match res {
            Ok(file) => all_files.push(file),
            // skip errors on individual files
            Err(_) => {}
        }
    } else if p.is_dir() {
        let all = read_dir(path)?;
        for item in all {
            let item = item?.path();
            if !rec && item.is_dir() {
                continue;
            }
            let res = parse_path(item, true);
            match res {
                Ok(mut files) => {
                    all_files.append(&mut files);
                }
                // skip errors in subdir
                Err(_) => {}
            }
        }
    } else if p.is_symlink() {
        bail!("Root folder cannot be a Symlink\n");
    } else {
        bail!("Unknown error\n");
    }
    if all_files.is_empty() {
        bail!("Music files was not found\n");
    }
    Ok(all_files)
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
    false
}
pub fn apply_selected(ui: &mut CoverUI, id: SongId) {
    let song = &mut ui.state.songs[id];
    let selected_img_hash = song.selected_img;
    let mut selected_img = None;
    for img in &mut song.imgs {
        if img.hash == selected_img_hash {
            selected_img = Some(img);
            break;
        }
    }
    if let Some(img) = selected_img {
        let (fin, fin_type, fin_prev) = img.final_img(&ui.state.img_settings);
        song.original_art = Some(fin_prev);

        let pic = Picture {
            data: &fin,
            mime_type: fin_type.audiotags(),
        };
        let tags = &mut song.tag_data.file;
        tags.set_album_cover(pic);
        tags.write_to_path(song.tag_data.path.to_str().unwrap())
            .unwrap();
    }
}
