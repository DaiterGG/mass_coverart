use anyhow::Error;
use iced::futures::channel::mpsc::Sender;
use log::info;
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
        img::{ImageProgress, ImgFormat, SongImg},
    },
};

pub struct Bandcamp {
    tags: TagsInput,
    tx: Sender<Message>,
    client: Client,
}
const SEARCH_LIMIT: i32 = 10;

impl WebSource for Bandcamp {
    fn build_title_pompt(&self, title: &str, artist: &str) -> String {
        format!("{} {}", artist, title)
    }
    fn build_album_pompt(&self, album: &str, artist: &str) -> String {
        format!("{} {}", artist, album)
    }
    const ALBUM_SOURCE: Source = BandcampAlbum;
    const TITLE_SOURCE: Source = BandcampTitle;

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
        let search_url = format!("https://bandcamp.com/search?q={}", encoded_query);

        info!("Fetching search: {}", search_url);

        let search_results_html = self.client.get(&search_url).send().await?.text().await?;

        dbg!(&search_results_html);
        // https://sourceforge.net/p/album-art/src/ci/main/tree/Scripts/Scripts/bandcamp.boo
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
                let full_size_url = if let Some(base) = Self::extract_base_image_url(&img_url) {
                    format!("{}_0.jpg", base)
                } else {
                    img_url.clone()
                };

                let thumb_url = if let Some(base) = Self::extract_base_image_url(&img_url) {
                    format!("{}_7.jpg", base)
                } else {
                    img_url.clone()
                };

                let feedback = format!("title: {}\nurl: {}", title, url);

                self.fetch_and_send_artwork(full_size_url, thumb_url, feedback, src)
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
impl Bandcamp {
    fn extract_base_image_url(img_url: &str) -> Option<String> {
        // Extract base URL from image URLs like:
        // https://f4.bcbits.com/img/a1816854455_7.jpg -> https://f4.bcbits.com/img/a1816854455
        let re = Regex::new(r"^(https?://[^/]+/img/[a-zA-Z0-9]+)_\d+\.jpg$").ok()?;
        let caps = re.captures(img_url)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }

    async fn fetch_and_send_artwork(
        &self,
        full_url: String,
        thumb_url: String,
        feedback: String,
        src: Source,
    ) -> Result<(), Error> {
        info!("Attempting to fetch bandcamp artwork from: {}", full_url);

        let image_data =
            shared::get_img(&self.client, vec![thumb_url.clone(), full_url.clone()]).await?;

        let song_img = SongImg::new(
            ImgFormat::Jpeg,
            ImageProgress::RawPreview(vec![full_url], image_data),
            src,
            feedback,
        );

        send_song(self, song_img).await;

        Ok(())
    }
}
