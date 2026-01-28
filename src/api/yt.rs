use anyhow::{Error, bail};
use iced::futures::channel::mpsc::Sender;
use log::{info, warn};
use reqwest::Client;
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
        img::{ImageProgress, ImgFormat::Jpeg, SongImg},
    },
};

pub async fn youtube(tags: TagsInput, tx: Sender<Message>) -> Result<(), Error> {
    let now = Instant::now();
    let client = Client::new();
    if tags.album.is_some() && tags.artist.is_some() {
        let prompt = format!(
            "{} {}",
            filter_for_query(tags.artist.as_ref().unwrap()),
            filter_for_query(tags.album.as_ref().unwrap()),
        );
        let _ = with_prompt(&tags, tx.clone(), &prompt, YoutubeAlbum, client.clone())
            .await
            .inspect_err(|e| warn!("request failed: {} {} {e}", tags.id, prompt));
    }
    if tags.title.is_some() && tags.artist.is_some() {
        let prompt = format!(
            "{} {} audio",
            filter_for_query(tags.artist.as_ref().unwrap()),
            filter_for_query(tags.title.as_ref().unwrap()),
        );
        with_prompt(&tags, tx.clone(), &prompt, YoutubeTitle, client.clone())
            .await
            .inspect_err(|e| warn!("request failed: {} {} {e}", tags.id, prompt))?;
    }

    info!("finished in {}ms", now.elapsed().as_millis());
    send_message(&tags, QueueMessage::SourceFinished, tx).await;
    Ok(())
}

const SEARCH_LIMIT: usize = 20;
async fn with_prompt(
    tags: &TagsInput,
    tx: Sender<Message>,
    prompt: &str,
    src: Source,
    client: Client,
) -> Result<(), Error> {
    let search = match YouTubeSearch::new(None, false) {
        Ok(search) => search,
        Err(e) => {
            bail!("Failed to initialize YouTubeSearch: {}", e);
        }
    };
    let filters = SearchFilters {
        sort_by: None,
        duration: None,
    };

    let results = search.search(prompt, filters).await.inspect_err(|e| {
        warn!("search failed {prompt}, {e}");
    })?;
    let mut limit = SEARCH_LIMIT;
    let mut i = 0;
    while i < limit && i < results.len() {
        if !results[i].duration.starts_with('0')
            && !results[i].duration.eq_ignore_ascii_case("1:00")
        {
            let _ = get_img(
                client.clone(),
                tags,
                results[i].video_id.clone(),
                results[i].title.clone(),
                results[i].channel_name.clone(),
                tx.clone(),
                src,
            )
            .await;
        } else {
            limit += 1;
        }
        i += 1;
    }

    Ok(())
}

async fn get_img(
    client: Client,
    tags: &TagsInput,
    link_id: String,
    title: String,
    channel: String,
    tx: Sender<Message>,
    src: Source,
) -> Result<(), Error> {
    let url_patterns = vec![
        format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", link_id),
        format!("https://img.youtube.com/vi/{}/hq720.jpg", link_id),
        // reasoning for sd order https://img.youtube.com/vi/h9eyp6cHnwM/sd2.jpg
        format!("https://img.youtube.com/vi/{}/sddefault.jpg", link_id),
        format!("https://img.youtube.com/vi/{}/sd2.jpg", link_id),
        format!("https://img.youtube.com/vi/{}/sd3.jpg", link_id),
    ];
    // mq can be in different aspect ratio that all other thumbnails versions for some reason
    // might pair it with sddefault since it also have different res
    let small_url = format!("https://img.youtube.com/vi/{}/mqdefault.jpg", link_id);
    // let small_url = format!("https://img.youtube.com/vi/{}/hqdefault.jpg", link_id);

    let pic = shared::get_img(&client, vec![small_url])
        .await
        .inspect_err(|e| {
            warn!("image could not download {link_id}, {e}");
        })?;

    let mut feedback = title;
    feedback.insert_str(0, "video title: ");
    feedback.push_str("\nchannel name: ");
    feedback.push_str(&channel);
    feedback.push_str("\nurl: https://www.youtube.com/watch?v=");
    feedback.push_str(&link_id);

    let new_img = SongImg::new(
        Jpeg,
        ImageProgress::RawPreview(url_patterns, pic),
        src,
        feedback,
    );
    send_song(new_img, tx.clone(), tags).await;

    Ok(())
}
