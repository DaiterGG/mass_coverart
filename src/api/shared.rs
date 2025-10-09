use anyhow::{Error, bail};
use bytes::Bytes;
use iced::futures::channel::mpsc::Sender;
use log::{info, warn};
use reqwest::Client;
use tokio::task::yield_now;

use crate::{
    api::queue::{QueueMessage, TagsInput},
    app::{iced_app::Message, img::SongImg},
};

pub async fn get_img(client: Client, urls: Vec<String>) -> Result<Bytes, Error> {
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
pub async fn send_message(tags: &TagsInput, mes: QueueMessage, mut tx: Sender<Message>) {
    let mut tried = tx.try_send(Message::FromQueue(tags.id, tags.hash, mes));
    while let Err(e) = tried
        && !e.is_full()
    {
        yield_now().await;
        tried = tx.try_send(e.into_inner());
    }
}
pub async fn send_song(img: SongImg, mut tx: Sender<Message>, tags: &TagsInput) {
    let mut tried = tx.try_send(Message::FromQueue(
        tags.id,
        tags.hash,
        QueueMessage::GotArt(img),
    ));
    while let Err(e) = tried
        && !e.is_full()
    {
        yield_now().await;
        tried = tx.try_send(e.into_inner());
    }
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
