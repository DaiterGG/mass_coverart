use std::fmt::Display;

use iced::{Task, futures::channel::mpsc::Sender, stream::channel, task::Handle, widget::image};
use log::{info, warn};
use tokio::task::JoinSet;

use crate::{
    api::{
        bandcamp::bandcamp,
        musicbrainz::musicbrainz,
        shared::send_message,
        yt::{self, youtube},
        yt_music::youtube_music,
    },
    app::{
        iced_app::Message,
        song::{SongHash, SongId},
        song_img::SongImg,
    },
    parser::file_parser::TagData,
};

#[derive(Clone, Copy, Debug)]
pub enum Source {
    YoutubeAlbum,
    YoutubeTitle,
    BrainzAlbum,
    BrainzTitle,
    BandcampAlbum,
    BandcampTitle,
    YoutubeMusicAlbum,
    YoutubeMusicTitle,
}
impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::YoutubeAlbum => write!(f, "youtube.com (%artist% %album% album)"),
            Self::YoutubeTitle => write!(f, "youtube.com (%artist% %title% audio)"),
            Self::BrainzAlbum => write!(f, "musicbrainz.com (%artist% %album%)"),
            Self::BrainzTitle => write!(f, "musicbrainz.com (%artist% %title%)"),
            Self::BandcampAlbum => write!(f, "bandcamp.com (%artist% %album%)"),
            Self::BandcampTitle => write!(f, "bandcamp.com (%artist% %title%)"),
            Self::YoutubeMusicAlbum => write!(f, "music.youtube.com (%artist% %album%)"),
            Self::YoutubeMusicTitle => write!(f, "music.youtube.com (%artist% %title%)"),
        }
    }
}

/// Song information to find album cover in queue
#[derive(Clone)]
pub struct TagsInput {
    pub id: SongId,
    pub hash: SongHash,
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
    GotArt(SongImg),
    SetSources(i32, i32),
    SourceFinished,
}

pub struct Queue;
impl Queue {
    pub fn init(tags: TagsInput) -> (Task<Message>, Handle) {
        Task::stream(channel(20, move |tx| Self::queue(tags, tx))).abortable()
    }
    pub const TOTAL_SOURCES: i32 = 4;
    async fn queue(tags: TagsInput, tx: Sender<Message>) {
        let mut set = JoinSet::new();
        set.spawn(musicbrainz(tags.clone(), tx.clone()));
        set.spawn(youtube_music(tags.clone(), tx.clone()));
        set.spawn(youtube(tags.clone(), tx.clone()));
        set.spawn(bandcamp(tags.clone(), tx.clone()));
        info!("queue is started for {}", tags.id);
        send_message(
            &tags,
            QueueMessage::SetSources(0, Self::TOTAL_SOURCES),
            tx.clone(),
        )
        .await;
        let logs = set.join_all().await;
        info!("queue is joined for {}", tags.id);

        for log in logs {
            let _ = log.inspect_err(|e| warn!("error occurred in queue of {} - {e}", tags.id));
        }
        send_message(
            &tags,
            QueueMessage::SetSources(Self::TOTAL_SOURCES, Self::TOTAL_SOURCES),
            tx,
        )
        .await;
    }
}
