use std::array::from_fn;

use crate::parser::file_parser::TagData;

pub const USER_INPUT_TAG_SCORE: i32 = 100;

#[derive(Debug, Clone, Copy)]
pub enum TagType {
    Artist,
    Album,
    Title,
    Total,
}
impl TagType {
    pub fn to_label(&self) -> &'static str {
        match self {
            TagType::Artist => "Artist",
            TagType::Album => "Album",
            TagType::Title => "Title",
            TagType::Total => panic!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectedTags {
    types: [Option<String>; TagType::Total as usize],
}
impl SelectedTags {
    pub fn new() -> Self {
        Self {
            types: from_fn(|_| None),
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn is_select(&self, key: TagType, value: &str) -> bool {
        if let Some(tag) = &self.types[key as usize]
            && tag == value
        {
            true
        } else {
            false
        }
    }
    pub fn select(&mut self, key: TagType, value: Option<String>) {
        self.types[key as usize] = value;
    }
    pub fn toggle(&mut self, key: TagType, value: Option<String>) {
        if self.types[key as usize] == value {
            self.types[key as usize] = None;
        } else {
            self.types[key as usize] = value;
        }
    }
    pub fn apply_selected(&mut self, tag_data: &mut TagData) -> Result<(), anyhow::Error> {
        let mut write = false;
        if let Some(album) = self.types[TagType::Album as usize].take() {
            write = true;
            tag_data.file.set_album_title(&album);
            tag_data.album = if album.is_empty() { None } else { Some(album) };
        }
        if let Some(title) = self.types[TagType::Title as usize].take() {
            write = true;
            tag_data.file.set_title(&title);
            tag_data.title = if title.is_empty() { None } else { Some(title) };
        }
        if let Some(artist) = self.types[TagType::Artist as usize].take() {
            write = true;
            tag_data.file.set_artist(&artist);
            tag_data.artist = if artist.is_empty() {
                None
            } else {
                Some(artist)
            };
        }
        if write {
            tag_data
                .file
                .write_to_path(tag_data.path.to_str().unwrap())?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Tag {
    pub score: i32,
    pub key: TagType,
    pub value: String,
}

#[derive(Default, Debug, Clone)]
pub struct Tags {
    pub sorted: Vec<Tag>,
}

impl Tags {
    pub fn new() -> Self {
        Self { sorted: Vec::new() }
    }
    pub fn extend(&mut self, new_tags: Vec<Tag>) {
        if new_tags.is_empty() {
            return;
        }
        for tag in new_tags {
            self.add_or_push(tag);
        }
        self.sort();
    }
    /// Add new tag or increase score of existing one
    pub fn add(&mut self, new_tag: Tag) {
        self.add_or_push(new_tag);
        self.sort();
    }
    fn add_or_push(&mut self, new_tag: Tag) {
        for tag in &mut self.sorted {
            if tag.value == new_tag.value {
                tag.score += new_tag.score;
                return;
            }
        }
        self.sorted.push(new_tag);
    }
    fn sort(&mut self) {
        self.sorted.sort_by(|a, b| a.score.cmp(&b.score));
    }
}
