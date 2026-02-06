use anyhow::Error;
use iced::futures::channel::mpsc::Sender;
use log::{info, warn};
use regex::RegexBuilder;
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
        img::{ImageProgress, ImgFormat, SongImg},
    },
};

pub struct Qobuz {
    tags: TagsInput,
    tx: Sender<Message>,
    client: Client,
}
const SEARCH_LIMIT: i32 = 10;

impl WebSource for Qobuz {
    fn build_title_pompt(&self, title: &str, artist: &str) -> String {
        format!("{} {}", artist, title)
    }
    fn build_album_pompt(&self, album: &str, artist: &str) -> String {
        format!("{} {}", artist, album)
    }
    const ALBUM_SOURCE: Source = QobuzAlbum;
    const TITLE_SOURCE: Source = QobuzTitle;

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
        let encoded_query: String = form_urlencoded::byte_serialize(prompt.as_bytes()).collect();
        let locale = "fr-fr";
        let search_url = format!(
            "http://www.qobuz.com/{}/search?i=boutique&q={}",
            locale, encoded_query
        );

        info!("Fetching search: {}", search_url);

        let search_results_html = self.client.get(&search_url).send().await?.text().await?;

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

                let feedback = format!("album title: {}\nurl: {}", title, url);
                self.fetch_and_send_artwork(img_small, img_base, feedback, src)
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
}
impl Qobuz {
    async fn fetch_and_send_artwork(
        &self,
        img_small: String,
        img_base: String,
        feedback: String,
        src: Source,
    ) -> Result<(), Error> {
        let url_patterns = vec![
            format!("{}_max.jpg", img_base),
            format!("{}_600.jpg", img_base),
        ];

        let thumbnail = shared::get_img(&self.client, vec![img_small.clone()])
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
        send_song(self, new_img).await;

        Ok(())
    }
}
