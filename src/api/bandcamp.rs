use anyhow::Error;
use iced::futures::channel::mpsc::Sender;
use log::{debug, info, warn};
use regex::Regex;
use reqwest::Client;
use tokio::time::Instant;

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
        song_img::{ImageProgress, ImgFormat, SongImg},
    },
};

const SEARCH_LIMIT: usize = 20;

pub async fn bandcamp(tags: TagsInput, tx: Sender<Message>) -> Result<(), Error> {
    let now = Instant::now();
    let client = Client::new();
    if tags.title.is_some() && tags.artist.is_some() {
        let query = format!(
            "\"{}\" \"{}\"",
            filter_for_query(tags.artist.as_ref().unwrap()),
            filter_for_query(tags.title.as_ref().unwrap()),
        );

        let _ = with_query(&tags, tx.clone(), &query, BandcampTitle, client.clone())
            .await
            .inspect_err(|e| warn!("request failed: {} {} {e}", tags.id, query));
    }
    if tags.album.is_some() && tags.artist.is_some() {
        let query = format!(
            "\"{}\" \"{}\"",
            filter_for_query(tags.artist.as_ref().unwrap()),
            filter_for_query(tags.album.as_ref().unwrap()),
        );

        with_query(&tags, tx.clone(), &query, BandcampAlbum, client.clone())
            .await
            .inspect_err(|e| warn!("request failed: {} {} {e}", tags.id, query))?;
    }

    info!("finished in {}ms", now.elapsed().as_millis());
    send_message(&tags, QueueMessage::SourceFinished, tx).await;
    Ok(())
}

async fn with_query(
    tags: &TagsInput,
    tx: Sender<Message>,
    query: &str,
    src: Source,
    client: Client,
) -> Result<(), Error> {
    let encoded_query: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let search_url = format!("https://bandcamp.com/search?q={}", encoded_query);

    info!("Fetching search: {}", search_url);

    let search_results_html = client.get(&search_url).send().await?.text().await?;

    // https://sourceforge.net/p/album-art/src/ci/main/tree/Scripts/Scripts/bandcamp.boo
    //    let re = Regex::new(r#"(?s)<li class="searchresult[^>]*>.*?<a class="artcont" href="([^"]+)".*?<img src="([^"]+)"[^>]*>.*?<div class="heading">\s*<a[^>]*>([^<]+)</a>"#)
    let re = Regex::new(r#"(?s)<li class="searchresult[^>]*>.*?<a class="artcont" href="([^"?]+)[^"]*".*?<img src="([^"]+)"[^>]*>.*?<div class="heading">\s*<a[^>]*>([^<]+)</a>"#)
        .map_err(|e| anyhow::anyhow!("Invalid regex: {}", e))?;

    let mut match_count = 0;

    for capture in re.captures_iter(&search_results_html) {
        if let (Some(url), Some(img_url), Some(title)) =
            (capture.get(1), capture.get(2), capture.get(3))
        {
            let url = url.as_str().to_string();
            let img_url = img_url.as_str().to_string();
            let title = title.as_str().trim_start().trim_end();

            info!("Found result: {} {}", title, url);
            info!("Image URL: {}", img_url);

            // Extract base image URL for higher quality versions
            let full_size_url = if let Some(base) = extract_base_image_url(&img_url) {
                format!("{}_0.jpg", base)
            } else {
                img_url.clone()
            };

            let thumb_url = if let Some(base) = extract_base_image_url(&img_url) {
                format!("{}_7.jpg", base)
            } else {
                img_url.clone()
            };

            let client_clone = client.clone();
            let tx_clone = tx.clone();
            let feedback = format!("title: {}\nurl: {}", title, url);

            fetch_and_send_artwork(
                client_clone,
                tags,
                tx_clone,
                full_size_url,
                thumb_url,
                feedback,
                src,
            )
            .await?;

            match_count += 1;
        }

        if match_count >= SEARCH_LIMIT {
            break;
        }
    }

    info!("Found {} matches in search", match_count);

    Ok(())
}

fn extract_base_image_url(img_url: &str) -> Option<String> {
    // Extract base URL from image URLs like:
    // https://f4.bcbits.com/img/a1816854455_7.jpg -> https://f4.bcbits.com/img/a1816854455
    let re = Regex::new(r"^(https?://[^/]+/img/[a-zA-Z0-9]+)_\d+\.jpg$").ok()?;
    let caps = re.captures(img_url)?;
    caps.get(1).map(|m| m.as_str().to_string())
}

async fn fetch_and_send_artwork(
    client: Client,
    tags: &TagsInput,
    tx: Sender<Message>,
    full_url: String,
    thumb_url: String,
    feedback: String,
    src: Source,
) -> Result<(), Error> {
    info!("Attempting to fetch bandcamp artwork from: {}", full_url);

    let image_data =
        shared::get_img(client.clone(), vec![thumb_url.clone(), full_url.clone()]).await?;

    let song_img = SongImg::new(
        ImgFormat::Jpeg,
        ImageProgress::RawPreview(vec![full_url], image_data),
        src,
        feedback,
    );

    send_song(song_img, tx.clone(), tags).await;

    Ok(())
}
