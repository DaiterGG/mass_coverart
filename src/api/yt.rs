use anyhow::{Error, bail};
use iced::futures::channel::mpsc::Sender;
use log::warn;
use reqwest::Client;
use yt_search::{SearchFilters, YouTubeSearch};

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

pub struct Youtube {
    tags: TagsInput,
    tx: Sender<Message>,
    client: Client,
}

const SEARCH_LIMIT: usize = 20;
impl WebSource for Youtube {
    fn build_title_pompt(&self, title: &str, artist: &str) -> String {
        format!("{} {} audio", artist, title)
    }
    fn build_album_pompt(&self, album: &str, artist: &str) -> String {
        format!("{} {}", artist, album)
    }
    const ALBUM_SOURCE: Source = YoutubeAlbum;
    const TITLE_SOURCE: Source = YoutubeTitle;

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
                let _ = self
                    .get_img(
                        results[i].video_id.clone(),
                        results[i].title.clone(),
                        results[i].channel_name.clone(),
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
}

impl Youtube {
    async fn get_img(
        &self,
        link_id: String,
        title: String,
        channel: String,
        src: Source,
    ) -> Result<(), Error> {
        let url_patterns = vec![
            format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", link_id),
            format!("https://img.youtube.com/vi/{}/hq720.jpg", link_id),
            // reasoning for sd order https://img.youtube.com/vi/h9eyp6cHnwM/sd2.jpg
            format!("https://img.youtube.com/vi/{}/sddefault.jpg", link_id),
            format!("https://img.youtube.com/vi/{}/sd2.jpg", link_id),
            format!("https://img.youtube.com/vi/{}/sd3.jpg", link_id),
            format!("https://img.youtube.com/vi/{}/hqdefault.jpg", link_id),
        ];
        // mq can be in different aspect ratio that all other thumbnails versions for some reason
        // might pair it with sddefault since it also have different res
        let small_url = format!("https://img.youtube.com/vi/{}/mqdefault.jpg", link_id);
        // let small_url = format!("https://img.youtube.com/vi/{}/hqdefault.jpg", link_id);

        let pic = shared::get_img(&self.client, vec![small_url])
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
        send_song(self, new_img).await;

        Ok(())
    }
}
