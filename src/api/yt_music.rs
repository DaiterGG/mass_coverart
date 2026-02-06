use std::collections::HashSet;

use anyhow::Error;
use iced::futures::channel::mpsc::Sender;
use log::{info, warn};
use regex::Regex;
use reqwest::Client;

use crate::{
    api::{
        queue::{
            Source::{self, *},
            TagsInput,
        },
        shared::{self, WebSource, send_song},
    },
    app::{
        iced_app::Message,
        img::{ImageProgress, ImgFormat::Jpeg, SongImg},
    },
};
pub struct YoutubeMus {
    tags: TagsInput,
    tx: Sender<Message>,
    client: Client,
}

const SEARCH_LIMIT: usize = 20;
impl WebSource for YoutubeMus {
    fn build_title_pompt(&self, title: &str, artist: &str) -> String {
        format!(r#""{}" "{}" album"#, artist, title)
    }
    fn build_album_pompt(&self, album: &str, artist: &str) -> String {
        format!(r#""{}" "{}""#, artist, album)
    }
    const ALBUM_SOURCE: Source = YoutubeMusAlbum;
    const TITLE_SOURCE: Source = YoutubeMusTitle;

    fn tags_ref(&self) -> &TagsInput {
        &self.tags
    }

    fn tx_ref(&self) -> &Sender<Message> {
        &self.tx
    }
    fn tx_clone(&self) -> Sender<Message> {
        self.tx.clone()
    }
    async fn init(tags: TagsInput, tx: Sender<Message>) -> Result<(), Error> {
        let this = Self {
            tags,
            tx,
            client: Client::new(),
        };
        shared::init_source(this).await?;
        Ok(())
    }
    async fn with_prompt(&self, prompt: &str, src: Source) -> Result<(), Error> {
        let search_url = format!("https://music.youtube.com/search?q={}", prompt);

        info!("Fetching youtube music search: {}", search_url);

        let search_results_html = self
            .client
            .get(&search_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:81.0) Gecko/20100101 Firefox/81.0",
            )
            .send()
            .await?
            .text()
            .await?;

        let re = Regex::new(
            r#"\\x22text\\x22:\\x22([^\\]+?)\\x22,\\x22navigationEndpoint.*?\\x22videoId\\x22:\\x22([A-Za-z0-9_-]{11})\\x22"#,
        )?;
        let mut results = Vec::new();
        let mut dedup = HashSet::new();
        for video_id in re.captures_iter(&search_results_html) {
            if let (Some(title), Some(id)) = (video_id.get(1), video_id.get(2)) {
                let title = title.as_str().to_string();
                let id = id.as_str().to_string();
                if !dedup.contains(&id) {
                    dedup.insert(id.clone());
                    results.push((title, id));
                } else {
                    results.last_mut().unwrap().0.push('\n');
                    results.last_mut().unwrap().0.push_str(&title);
                }
            }
        }

        let mut i = 0;
        while i < SEARCH_LIMIT && i < results.len() {
            let _ = self
                .get_img(results[i].0.clone(), results[i].1.clone(), src)
                .await;
            i += 1;
        }

        Ok(())
    }
}
impl YoutubeMus {
    async fn get_img(&self, title: String, link_id: String, src: Source) -> Result<(), Error> {
        let url_patterns = vec![
            format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", link_id),
            format!("https://img.youtube.com/vi/{}/hq720.jpg", link_id),
            format!("https://img.youtube.com/vi/{}/sddefault.jpg", link_id),
            format!("https://img.youtube.com/vi/{}/sd2.jpg", link_id),
            format!("https://img.youtube.com/vi/{}/sd3.jpg", link_id),
            format!("https://img.youtube.com/vi/{}/hqdefault.jpg", link_id),
        ];
        let small_url = format!("https://img.youtube.com/vi/{}/mqdefault.jpg", link_id);

        let pic = shared::get_img(&self.client, vec![small_url])
            .await
            .inspect_err(|e| {
                warn!("image could not download {link_id}, {e}");
            })?;

        // title scraping is inconsistent
        // feedback.insert_str(0, "title: ");
        let mut feedback = "info: ".to_string();
        feedback.push_str(&title);
        feedback.push_str("\nurl: https://www.youtube.com/watch?v=");
        feedback.push_str(&link_id);

        let new_img = SongImg::new(
            Jpeg,
            ImageProgress::RawPreview(url_patterns, pic),
            src,
            feedback,
        );
        send_song(self, new_img).await;

        Ok(())
    }
}
