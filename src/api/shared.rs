use std::time::Instant;

use anyhow::{Error, bail};
use bytes::Bytes;
use iced::futures::channel::mpsc::Sender;
use log::{info, warn};
use reqwest::Client;
use tokio::task::yield_now;

use crate::{
    api::queue::{QueueMessage, Source, TagsInput},
    app::{iced_app::Message, img::SongImg, tags::Tags},
};

pub trait WebSource {
    const ALBUM_SOURCE: Source;
    const TITLE_SOURCE: Source;
    async fn init(tags: TagsInput, tx: Sender<Message>) -> Result<(), Error>;
    fn tags_ref(&self) -> &TagsInput;
    fn tx_ref(&self) -> &Sender<Message>;
    fn tx_clone(&self) -> Sender<Message>;
    fn build_title_pompt(&self, title: &str, artist: &str) -> String;
    fn build_album_pompt(&self, album: &str, artist: &str) -> String;
    async fn with_prompt(&self, prompt: &str, src: Source) -> Result<(), Error>;
}

pub async fn init_source<T: WebSource>(src: T) -> Result<(), Error> {
    let now = Instant::now();
    if let Some(ref album) = src.tags_ref().album
        && let Some(ref artist) = src.tags_ref().artist
    {
        let prompt = src.build_album_pompt(&album, &artist);
        let _ = src
            .with_prompt(&prompt, T::ALBUM_SOURCE)
            .await
            .inspect_err(|e| warn!("request failed: {} {} {e}", src.tags_ref().id, prompt));
    }
    if let Some(ref title) = src.tags_ref().title
        && let Some(ref artist) = src.tags_ref().artist
    {
        let prompt = src.build_title_pompt(&title, &artist);
        src.with_prompt(&prompt, T::TITLE_SOURCE)
            .await
            .inspect_err(|e| warn!("request failed: {} {} {e}", src.tags_ref().id, prompt))?;
    }

    info!("finished in {}ms", now.elapsed().as_millis());
    send_message_from_source(&src, QueueMessage::SourceFinished).await;
    Ok(())
}

pub async fn send_message(tags: &TagsInput, tx: &mut Sender<Message>, mes: QueueMessage) {
    let id = tags.id;
    let hash = tags.hash;
    let mut tried = tx.try_send(Message::FromQueue(id, hash, mes));
    while let Err(e) = tried
        && !e.is_full()
    {
        yield_now().await;
        tried = tx.try_send(e.into_inner());
    }
}
pub async fn send_message_from_source<T: WebSource>(src: &T, mes: QueueMessage) {
    send_message(src.tags_ref(), &mut src.tx_clone(), mes).await;
}

pub async fn send_song<T: WebSource>(src: &T, img: SongImg) {
    send_message_from_source(src, QueueMessage::GotArt(img)).await
}
pub async fn get_img(client: &Client, urls: Vec<String>) -> Result<Bytes, Error> {
    let mut last_error = None;
    for url in &urls {
        info!("Trying to get img: {}", url);
        let req = client.get(url).build()?;

        match client.execute(req).await {
            Ok(response) => {
                info!("Getting img body, url: {}", response.url());
                let pic = response.bytes().await?;
                if pic.len() == 1097 {
                    last_error = Some(anyhow::Error::msg("\"No image\" received"));
                    continue;
                }
                return Ok(pic);
            }
            Err(e) => {
                warn!("failed to get img {url}");
                last_error = Some(Error::new(e));
            }
        }
    }

    bail!(last_error.unwrap())
}

pub fn filter_for_query(string: &str) -> String {
    string
        .chars()
        .filter(|c| {
            *c != '&'
                || *c != '.'
                || *c != '\''
                || *c != '\\'
                || *c != '"'
                || *c != ';'
                || *c != ':'
                || *c != '?'
                || *c != '!'
        })
        .collect::<String>()
}
