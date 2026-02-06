use anyhow::Error;
use iced::futures::channel::mpsc::Sender;
use log::info;
use musicbrainz_rs::{
    FetchCoverart, MusicBrainzClient, Search,
    entity::{
        CoverartResponse,
        release::{Release, ReleaseSearchQuery},
    },
};

use crate::api::{
    queue::{
        Source::{self, *},
        TagsInput,
    },
    shared::{self, WebSource, send_song},
};
use crate::app::{
    iced_app::Message,
    img::{
        ImageProgress::*,
        ImgFormat::{self},
        SongImg,
    },
};

pub struct Musicbrainz {
    tags: TagsInput,
    tx: Sender<Message>,
    b_client: MusicBrainzClient,
}

const SEARCH_LIMIT: usize = 20;
impl WebSource for Musicbrainz {
    fn build_title_pompt(&self, title: &str, artist: &str) -> String {
        ReleaseSearchQuery::query_builder()
            .artist(title)
            .and()
            .release(artist)
            .build()
    }
    fn build_album_pompt(&self, album: &str, artist: &str) -> String {
        ReleaseSearchQuery::query_builder()
            .artist(album)
            .and()
            .release(artist)
            .build()
    }
    const ALBUM_SOURCE: Source = BrainzAlbum;
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
            b_client: MusicBrainzClient::default(),
        };
        shared::init_source(this).await?;
        Ok(())
    }
    async fn with_prompt(&self, query: &str, src: Source) -> Result<(), Error> {
        // TODO: rate limit throws internal crate error
        let query_result = Release::search(query.to_string())
            .execute_with_client(&self.b_client)
            .await?;

        info!(
            "Found {} matches in song {}",
            query_result.count, self.tags.id
        );

        for release in query_result.entities {
            let artists = if let Some(vec) = release.artist_credit {
                let mut line = "".to_string();
                for artist in vec {
                    line.push_str(&artist.name);
                }
                line
            } else {
                "unknown".to_string()
            };
            let feedback = format!("artist: {}, release: {},", artists, release.title);

            let releases_result = Release::fetch_coverart()
                .id(&release.id)
                .execute_with_client(&self.b_client)
                .await;

            let client = &self.b_client.reqwest_client;
            if let Ok(cover_response) = releases_result
                && let CoverartResponse::Json(cover) = cover_response
            {
                for img in cover.images {
                    let new_song = if let Some(thumb) = img.thumbnails.res_250 {
                        let res = shared::get_img(client, vec![thumb]).await;
                        if res.is_err() {
                            continue;
                        }
                        RawPreview(vec![img.image.clone()], res.unwrap())
                    } else if let Some(thumb) = img.thumbnails.small {
                        let res = shared::get_img(client, vec![thumb]).await;
                        if res.is_err() {
                            continue;
                        }
                        RawPreview(vec![img.image.clone()], res.unwrap())
                    } else {
                        let res = shared::get_img(client, vec![img.image.clone()]).await;
                        if res.is_err() {
                            continue;
                        }
                        info!("only full picture available for {:?}", self.tags.hash);
                        Raw(res.unwrap())
                    };
                    let new_img = SongImg::new(
                        ImgFormat::from_url(&img.image),
                        new_song,
                        src,
                        feedback.clone(),
                    );
                    send_song(self, new_img).await;
                }
            }
        }

        Ok(())
    }
}
