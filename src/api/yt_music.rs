use std::collections::{HashMap, HashSet};

use anyhow::{Error, bail};
use iced::futures::channel::mpsc::Sender;
use log::{info, warn};
use regex::{Captures, Regex};
use reqwest::Client;
use serde_json::to_string;
use tokio::time::Instant;
use yt_search::{SearchFilters, YouTubeSearch};

use crate::{
    api::{
        queue::{
            QueueMessage,
            Source::{self, *},
            TagsInput,
        },
        shared::{self, filter_for_query, send_message, send_song},
    },
    app::{
        iced_app::Message,
        song_img::{ImgFormat::Jpg, LazyImage, SongImg},
    },
};

const SEARCH_LIMIT: usize = 20;
pub async fn youtube_music(tags: TagsInput, tx: Sender<Message>) -> Result<(), Error> {
    let now = Instant::now();
    let client = Client::new();
    if tags.album.is_some() && tags.artist.is_some() {
        let prompt = format!(
            r#""{}" "{}" album"#,
            filter_for_query(tags.artist.as_ref().unwrap()),
            filter_for_query(tags.album.as_ref().unwrap()),
        );
        let _ = with_prompt(
            &tags,
            tx.clone(),
            &prompt,
            YoutubeMusicAlbum,
            client.clone(),
        )
        .await
        .inspect_err(|e| warn!("request failed: {} {} {e}", tags.id, prompt));
    }
    if tags.title.is_some() && tags.artist.is_some() {
        let prompt = format!(
            r#""{}" "{}""#,
            filter_for_query(tags.artist.as_ref().unwrap()),
            filter_for_query(tags.title.as_ref().unwrap()),
        );
        with_prompt(
            &tags,
            tx.clone(),
            &prompt,
            YoutubeMusicTitle,
            client.clone(),
        )
        .await
        .inspect_err(|e| warn!("request failed: {} {} {e}", tags.id, prompt))?;
    }

    info!("finished in {}ms", now.elapsed().as_millis());
    send_message(&tags, QueueMessage::SourceFinished, tx).await;
    Ok(())
}

async fn with_prompt(
    tags: &TagsInput,
    tx: Sender<Message>,
    prompt: &str,
    src: Source,
    client: Client,
) -> Result<(), Error> {
    let search_url = format!("https://music.youtube.com/search?q={}", prompt);

    info!("Fetching youtube music search: {}", search_url);

    let search_results_html = client
        .get(&search_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:81.0) Gecko/20100101 Firefox/81.0",
        )
        .send()
        .await?
        .text()
        .await?;

    let mut dedup = HashSet::new();
    let mut results = Vec::new();
    let re = Regex::new(r#"\\x22watchEndpoint\\x22:\\x7b\\x22videoId\\x22:\\x22([^\\]{11})"#)?;
    for video_id in re.captures_iter(&search_results_html) {
        if let Some(mtch) = video_id.get(0) {
            let video_id = mtch.as_str().rsplit_once("\\x22").unwrap().1;
            if !dedup.contains(&video_id) {
                results.push(video_id);
                dedup.insert(video_id);
            }
        }
    }

    let mut i = 0;
    while i < SEARCH_LIMIT && i < results.len() {
        let _ = get_img(
            client.clone(),
            tags,
            results[i].to_string(),
            // results[i].title.clone(),
            tx.clone(),
            src,
        )
        .await;
        i += 1;
    }

    Ok(())
}

async fn get_img(
    client: Client,
    tags: &TagsInput,
    link_id: String,
    // mut feedback: String,
    tx: Sender<Message>,
    src: Source,
) -> Result<(), Error> {
    let url_patterns = vec![
        format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", link_id),
        format!("https://img.youtube.com/vi/{}/hq720.jpg", link_id),
        format!("https://img.youtube.com/vi/{}/sddefault.jpg", link_id),
    ];
    let small_url = format!("https://img.youtube.com/vi/{}/mqdefault.jpg", link_id);

    let pic = shared::get_img(client, vec![small_url])
        .await
        .inspect_err(|e| {
            warn!("image could not download {link_id}, {e}");
        })?;
    if pic.len() == 1097 {
        info!("\"No image\" received");
        return Ok(());
    }

    // feedback.insert_str(0, "title: ");

    let new_img = SongImg::new(
        Jpg,
        LazyImage::RawPreview(url_patterns, pic),
        src,
        "".to_string(),
    );
    send_song(new_img, tx.clone(), tags).await;

    Ok(())
}
