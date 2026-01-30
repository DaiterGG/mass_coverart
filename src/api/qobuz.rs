use core::time;

use anyhow::Error;
use iced::futures::channel::mpsc::Sender;
use log::{debug, info, warn};
use regex::{Regex, RegexBuilder};
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
        img::{ImageProgress, ImgFormat, SongImg},
    },
};

const SEARCH_LIMIT: i32 = 10;
pub async fn qobuz(tags: TagsInput, tx: Sender<Message>) -> Result<(), Error> {
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

        with_query(&tags, tx.clone(), &query, QobuzAlbum, client.clone())
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
    let locale = "fr-fr";
    let search_url = format!(
        "http://www.qobuz.com/{}/search?i=boutique&q={}",
        locale, encoded_query
    );

    info!("Fetching search: {}", search_url);

    let search_results_html = client.get(&search_url).send().await?.text().await?;

    let re = RegexBuilder::new(r#"<div class="ReleaseCard">\s*<img\s*class="CoverModel"\s*src="(?<thumb>(?<imgBase>[^_]+)[^\"]+)[^>]+>.+?<a\s*class="ReleaseCardInfosTitle"\s*href="(?<url>[^\"]+)"[^>]+data-title="(?<title>[^\"]+)""#)
        .multi_line(true)
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .map_err(|e| anyhow::anyhow!("Invalid regex: {}", e))?;

    let mut match_count = 0;

    for capture in re.captures_iter(&search_results_html) {
        if let (Some(img_url), Some(img_base), Some(url_end), Some(title)) = (
            capture.get(1),
            capture.get(2),
            capture.get(3),
            capture.get(4),
        ) {
            let title = title.as_str().to_string();
            let mut url = "http://www.qobuz.com/".to_string();
            url.push_str(url_end.as_str());
            let img_small = img_url.as_str().to_string();
            let img_base = img_base.as_str().to_string();

            info!("Found result: {} {}", title, url);
            info!("Image URL: {}", img_base);

            let client_clone = client.clone();
            let tx_clone = tx.clone();

            let feedback = format!("album title: {}\nurl: {}", title, url);
            fetch_and_send_artwork(
                client_clone,
                tags,
                img_small,
                img_base,
                feedback,
                tx_clone,
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

async fn fetch_and_send_artwork(
    client: Client,
    tags: &TagsInput,
    img_small: String,
    img_base: String,
    feedback: String,
    tx: Sender<Message>,
    src: Source,
) -> Result<(), Error> {
    let url_patterns = vec![
        format!("{}_max.jpg", img_base),
        format!("{}_600.jpg", img_base),
    ];

    let thumbnail = shared::get_img(&client, vec![img_small.clone()])
        .await
        .inspect_err(|e| {
            warn!("image thumbnail could not download {img_small}, {e}");
        })?;

    let new_img = SongImg::new(
        ImgFormat::Jpeg,
        ImageProgress::RawPreview(url_patterns, thumbnail),
        src,
        feedback,
    );
    send_song(new_img, tx.clone(), tags).await;

    Ok(())
}
