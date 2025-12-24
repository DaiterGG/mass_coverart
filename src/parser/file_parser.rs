use anyhow::{Error, bail};
use iced::Length::Fill;
use log::info;
use rfd::FileHandle;
use std::{
    fmt::Debug,
    fs::{read, read_dir},
    path::{Path, PathBuf},
    time::Instant,
};

use audiotags::{AudioTag, Picture};

use crate::app::{
    iced_app::CoverUI,
    song::{OrigArt, Song, SongId},
    tags::{Tag, TagType, Tags, USER_INPUT_TAG_SCORE},
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
    fn reg_to_tags(&self, entry: &str, data: &mut TagData) -> Option<Tag> {
        match self {
            Self::Album => {
                if let Some(e) = &data.album
                    && entry == e
                {
                    return None;
                };
                data.album.get_or_insert(entry.to_string());
                Some(Tag {
                    value: entry.to_string(),
                    score: USER_INPUT_TAG_SCORE,
                    key: TagType::Album,
                })
            }
            Self::Title => {
                if let Some(e) = &data.title
                    && entry == e
                {
                    return None;
                };
                data.title.get_or_insert(entry.to_string());
                Some(Tag {
                    value: entry.to_string(),
                    score: USER_INPUT_TAG_SCORE,
                    key: TagType::Title,
                })
            }
            Self::Artist => {
                if let Some(e) = &data.artist
                    && entry == e
                {
                    return None;
                };
                data.artist.get_or_insert(entry.to_string());
                Some(Tag {
                    value: entry.to_string(),
                    score: USER_INPUT_TAG_SCORE,
                    key: TagType::Artist,
                })
            }
            Self::None => None,
        }
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
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
) -> Result<Vec<Song>, Error> {
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
fn map_tag(src: Option<&str>) -> Option<String> {
    if let Some(s) = src {
        if s == "" { None } else { Some(s.to_string()) }
    } else {
        None
    }
}

pub fn parse_tags(song: &mut Song, set: &ParseSettings) {
    let tags = &mut song.tag_data;
    tags.artist = map_tag(tags.file.artist());
    tags.title = map_tag(tags.file.title());
    tags.album = map_tag(tags.file.album_title());
    if !set.parse_file_name {
        return;
    }
    let mut remaider = tags.path.file_stem().unwrap().to_string_lossy().to_string();
    for i in 0..set.reg_separators.len() {
        let sep = &set.reg_separators[i];
        if let Some((entry, rest)) = remaider.split_once(sep) {
            let new_tag = set.reg_keys[i].reg_to_tags(entry, tags);
            if let Some(t) = new_tag {
                song.tags_from_regex.push(t);
            }
            remaider = rest.to_string();
        }
    }
    let new_tag = set
        .reg_keys
        .last()
        .expect("at least one regex")
        .reg_to_tags(&remaider, tags);

    if let Some(t) = new_tag {
        song.tags_from_regex.push(t);
    }
}
pub fn parse_path(path: PathBuf, rec: bool) -> Result<Vec<Song>, Error> {
    let mut all_files = Vec::new();
    let p: &Path = path.as_ref();
    if p.is_file() {
        let res = parse_file(path);
        if let Ok(file) = res {
            all_files.push(file)
        }
    } else if p.is_dir() {
        let all = read_dir(path)?;
        for item in all {
            let item = item?.path();
            if !rec && item.is_dir() {
                continue;
            }
            let res = parse_path(item, true);
            if let Ok(mut files) = res {
                all_files.append(&mut files);
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
pub fn parse_file(path: PathBuf) -> Result<Song, Error> {
    let file = audiotags::Tag::new().read_from_path(&path)?;
    Ok(Song::new(TagData::new(path, file)))
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
pub fn apply_selected(ui: &mut CoverUI, id: SongId) -> Result<(), Error> {
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
        info!("final img {}", img.image.dbg());
        let (fin, fin_type, fin_prev) = img.final_img(&ui.state.img_settings);
        song.original_art = Some(OrigArt::Loaded(fin_prev));

        let pic = Picture {
            data: &fin,
            mime_type: fin_type.audiotags(),
        };
        let tags = &mut song.tag_data.file;
        tags.set_album_cover(pic);
        tags.write_to_path(song.tag_data.path.to_str().unwrap())?;
    }
    Ok(())
}
pub fn find_edited_tags(tag_data: &TagData) -> Vec<Tag> {
    let title = map_tag(tag_data.file.title());
    let artist = map_tag(tag_data.file.artist());
    let album = map_tag(tag_data.file.album_title());

    let mut tags = Vec::new();
    if tag_data.title != title {
        tags.push(Tag {
            key: TagType::Title,
            value: tag_data.title.clone().unwrap_or_default(),
            score: USER_INPUT_TAG_SCORE,
        });
    }
    if tag_data.artist != artist {
        tags.push(Tag {
            key: TagType::Artist,
            value: tag_data.artist.clone().unwrap_or_default(),
            score: USER_INPUT_TAG_SCORE,
        });
    }
    if tag_data.album != album {
        tags.push(Tag {
            key: TagType::Album,
            value: tag_data.album.clone().unwrap_or_default(),
            score: USER_INPUT_TAG_SCORE,
        });
    }
    tags
}
