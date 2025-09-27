use std::fmt::Display;

use iced::{Task, futures::channel::mpsc::Sender, stream::channel, task::Handle, widget::image};

use crate::{
    api::yt,
    app::{
        iced_app::Message,
        song::{SongHash, SongId},
        song_img::SongImg,
    },
    parser::file_parser::TagData,
};

#[derive(Clone, Debug)]
pub enum Source {
    Youtube,
}
impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Youtube => write!(f, "youtube.com"),
        }
    }
}
/// return SongImg from queue
#[derive(Clone, Debug)]
pub struct ReturnSongImg {
    pub id: SongId,
    pub hash: SongHash,
    pub img: SongImg,
}
impl ReturnSongImg {
    pub fn new(id: SongId, hash: SongHash, img: SongImg) -> Self {
        Self { id, hash, img }
    }
    pub fn from_input(input: &TagsInput, img: SongImg) -> Self {
        Self {
            id: input.id,
            hash: input.hash,
            img,
        }
    }
}

/// Song information to find album cover in queue
#[derive(Clone)]
pub struct TagsInput {
    pub id: SongId,
    pub hash: u64,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
}
impl TagsInput {
    pub fn from_data(id: SongId, hash: u64, data: &TagData) -> Self {
        Self {
            id,
            hash,
            artist: data.artist.clone(),
            title: data.title.clone(),
            album: data.album.clone(),
        }
    }
}
#[derive(Clone, Debug)]
pub enum QueueMessage {
    GotArt(ReturnSongImg),
}
pub struct Queue;
impl Queue {
    pub fn init(tags: TagsInput) -> (Task<Message>, Handle) {
        Task::stream(channel(20, move |tx| Self::queue(tags, tx))).abortable()
    }
    async fn queue(tags: TagsInput, tx: Sender<Message>) {
        let h = yt::youtube(tags.clone(), tx.clone());
        let mut active = 1;
        h.await;
        active -= 1;
    }
}
