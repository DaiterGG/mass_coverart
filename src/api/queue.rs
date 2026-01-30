use std::fmt::Display;

use iced::{Task, futures::channel::mpsc::Sender, stream::channel, task::Handle, widget::image};
use log::{info, warn};
use tokio::task::JoinSet;

use crate::{
    api::{
        bandcamp::bandcamp,
        musicbrainz::musicbrainz,
        qobuz::qobuz,
        shared::send_message,
        yt::{self, youtube},
        yt_music::youtube_music,
    },
    app::{
        iced_app::Message,
        img::SongImg,
        song::{SongHash, SongId},
    },
    parser::file_parser::TagData,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    LocalFile,
    YoutubeAlbum,
    YoutubeTitle,
    BrainzAlbum,
    BrainzTitle,
    BandcampAlbum,
    BandcampTitle,
    QobuzTitle,
    QobuzAlbum,
    YoutubeMusicAlbum,
    YoutubeMusicTitle,
}
impl Source {
    pub fn get_weight(&self) -> i32 {
        match self {
            // Local files always on top
            Self::LocalFile => 9999,
            Self::BrainzTitle => 30,
            Self::BrainzAlbum => 30,
            Self::BandcampAlbum => 15,
            Self::BandcampTitle => 15,
            Self::QobuzTitle => 15,
            Self::QobuzAlbum => 15,
            _ => 10,
        }
    }
}
impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LocalFile => write!(f, "local file"),
            Self::YoutubeAlbum => write!(f, "youtube.com (%artist% %album% album)"),
            Self::YoutubeTitle => write!(f, "youtube.com (%artist% %title% audio)"),
            Self::BrainzAlbum => write!(f, "musicbrainz.com (%artist% %album%)"),
            Self::BrainzTitle => write!(f, "musicbrainz.com (%artist% %title%)"),
            Self::BandcampAlbum => write!(f, "bandcamp.com (%artist% %album%)"),
            Self::BandcampTitle => write!(f, "bandcamp.com (%artist% %title%)"),
            Self::QobuzTitle => write!(f, "qobuz.com (%artist% %title%)"),
            Self::QobuzAlbum => write!(f, "qobuz.com (%artist% %album%)"),
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
        // set.spawn(musicbrainz(tags.clone(), tx.clone()));
        // set.spawn(youtube_music(tags.clone(), tx.clone()));
        // set.spawn(youtube(tags.clone(), tx.clone()));
        // set.spawn(bandcamp(tags.clone(), tx.clone()));
        set.spawn(qobuz(tags.clone(), tx.clone()));
        info!("queue is started for {}", tags.id);
        send_message(
            &tags,
            QueueMessage::SetSources(0, Self::TOTAL_SOURCES),
            tx.clone(),
        )
        .await;

        while let Some(res) = set.join_next().await {
            let _ = res.inspect_err(|e| warn!("error occurred in queue of {} - {e}", tags.id));
        }
        info!("queue is joined for {}", tags.id);
        send_message(
            &tags,
            QueueMessage::SetSources(Self::TOTAL_SOURCES, Self::TOTAL_SOURCES),
            tx,
        )
        .await;
    }
}
