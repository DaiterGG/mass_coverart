use bytes::Bytes;
use iced::{Task, futures::channel::mpsc::Sender, stream::channel, widget::image::Handle};

use crate::{
    api::yt,
    app::{iced_app::Message, song::SongId},
    parser::file_parser::{FileData, TagData},
};

#[derive(Clone, Debug)]
pub enum ImgFormat {
    Jpeg,
    Png,
}
#[derive(Clone, Debug)]
pub struct Art {
    pub id: SongId,
    pub img: ImgData,
}
#[derive(Clone, Debug)]
pub enum ImgData {
    Path(String),
    PreviewPathUrl(String, String),
    Bytes(Handle, ImgFormat),
}
#[derive(Clone)]
pub struct TagsInput {
    pub id: SongId,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
}
impl TagsInput {
    pub fn from_data(id: SongId, data: &TagData) -> Self {
        Self {
            id,
            artist: data.artist.clone(),
            title: data.title.clone(),
            album: data.album.clone(),
        }
    }
}
pub struct Queue;
impl Queue {
    pub fn init(tags: TagsInput) -> Task<Message> {
        Task::stream(channel(20, move |tx| Self::queue(tags, tx)))
    }
    async fn queue(tags: TagsInput, tx: Sender<Message>) {
        let h = yt::youtube(tags.clone(), tx.clone());
        let mut active = 1;
        h.await;
        active -= 1;
    }
}
