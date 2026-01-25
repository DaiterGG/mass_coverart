use rand::RngCore;

use crate::{
    ImgHandle, TaskHandle,
    api::queue::Queue,
    app::{
        img::{ImgId, SongImg},
        img_group::ImgGroups,
        tags::{SelectedTags, Tag, Tags},
    },
    parser::file_parser::TagData,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SongState {
    Confirm,
    Main,
    MainLoading,
    MainDownloading,
    Hidden,
}
impl SongState {
    pub fn state_to_h(&self) -> f32 {
        use crate::app::song_view::CONFIRM_H;
        use crate::app::song_view::MAIN_H;
        match self {
            SongState::Confirm => CONFIRM_H,
            SongState::Main => MAIN_H,
            SongState::MainLoading => MAIN_H,
            SongState::MainDownloading => MAIN_H,
            _ => 0.0,
        }
    }
}
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum OrigArt {
    Unloaded,
    Loading,
    Loaded(ImgHandle),
}

/// Hash to check, when async queue return data
pub type SongHash = u64;
pub type SongId = usize;
#[derive(Debug, Clone)]
/// * `sources_finished`: x out of y
/// * `imgs`: only push() or empty()
/// * `tags_from_regex`: tags from regex to add to new_tags list
/// * every time confirm is pressed
pub struct Song {
    pub tag_data: TagData,
    pub state: SongState,
    pub queue_handle: Option<TaskHandle>,
    pub original_art: Option<OrigArt>,
    pub hash: SongHash,
    pub menu_img: Option<ImgId>,
    pub selected_img: Option<ImgId>,
    pub sources_finished: (i32, i32),
    pub img_groups: ImgGroups,
    pub imgs: Vec<SongImg>,

    pub new_tags: Tags,
    pub tags_from_regex: Vec<Tag>,
    pub selected_tags: SelectedTags,
}

impl Song {
    pub fn new(tag_data: TagData) -> Self {
        let original_art = if tag_data.file.album_cover().is_some() {
            Some(OrigArt::Unloaded)
        } else {
            None
        };
        Self {
            state: SongState::Confirm,
            queue_handle: None,
            original_art,
            tag_data,
            hash: rand::rng().next_u64(),
            menu_img: None,
            selected_img: None,
            sources_finished: (0, Queue::TOTAL_SOURCES),
            img_groups: ImgGroups::new(),
            imgs: Vec::new(),
            new_tags: Tags::new(),
            tags_from_regex: Vec::new(),
            selected_tags: SelectedTags::new(),
        }
    }

    pub fn reset(&mut self) {
        self.queue_handle.take().unwrap().abort();
        self.imgs.clear();
        self.img_groups.clear();
        self.selected_img = None;
        self.menu_close();
        self.selected_tags.reset();
        self.new_tags.sorted.clear();
    }
    pub fn menu_close(&mut self) {
        self.menu_img = None;
    }
}
