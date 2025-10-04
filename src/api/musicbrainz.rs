use anyhow::{Error, bail};
use iced::futures::channel::mpsc::Sender;
use log::{info, warn};
use musicbrainz_rs::{
    FetchCoverart, MusicBrainzClient, Search,
    entity::{
        CoverartResponse,
        release::{Release, ReleaseSearchQuery},
    },
};
use tokio::time::Instant;

use crate::api::{
    queue::{
        QueueMessage,
        Source::{self, *},
        TagsInput,
    },
    shared::{self, send_message, send_song},
};
use crate::app::{
    iced_app::Message,
    song_img::{
        ImageProgress::*,
        ImgFormat::{self},
        SongImg,
    },
};

pub async fn musicbrainz(tags: TagsInput, tx: Sender<Message>) -> Result<(), Error> {
    let now = Instant::now();
    let client = MusicBrainzClient::default();
    if tags.album.is_some() && tags.artist.is_some() {
        let query = ReleaseSearchQuery::query_builder()
            .artist(tags.artist.as_ref().unwrap())
            .and()
            .release(tags.album.as_ref().unwrap())
            .build();

        let _ = with_query(&tags, tx.clone(), &query, BrainzAlbum, &client)
            .await
            .inspect_err(|e| warn!("request failed: {} {query} {e}", tags.id));
    }

    if tags.title.is_some() && tags.artist.is_some() {
        let query = ReleaseSearchQuery::query_builder()
            .artist(tags.artist.as_ref().unwrap())
            .and()
            .release(tags.title.as_ref().unwrap())
            .build();

        with_query(&tags, tx.clone(), &query, BrainzTitle, &client)
            .await
            .inspect_err(|e| warn!("request failed: {} {query} {e}", tags.id))?;
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
    b_client: &MusicBrainzClient,
) -> Result<(), Error> {
    let query_result = Release::search(query.to_string())
        .execute_with_client(b_client)
        .await?;

    info!("Found {} matches in song {}", query_result.count, tags.id);

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
            .execute_with_client(b_client)
            .await;

        let client = b_client.get_reqwest_client();
        if let Ok(cover_response) = releases_result
            && let CoverartResponse::Json(cover) = cover_response
        {
            for img in cover.images {
                let new_song = if let Some(thumb) = img.thumbnails.res_250 {
                    let res = shared::get_img(client.clone(), vec![thumb]).await;
                    if res.is_err() {
                        continue;
                    }
                    RawPreview(vec![img.image.clone()], res.unwrap())
                } else if let Some(thumb) = img.thumbnails.small {
                    let res = shared::get_img(client.clone(), vec![thumb]).await;
                    if res.is_err() {
                        continue;
                    }
                    RawPreview(vec![img.image.clone()], res.unwrap())
                } else {
                    let res = shared::get_img(client.clone(), vec![img.image.clone()]).await;
                    if res.is_err() {
                        continue;
                    }
                    info!("only full picture available for {:?}", tags.hash);
                    Raw(res.unwrap())
                };
                let new_img = SongImg::new(
                    ImgFormat::from_url(&img.image),
                    new_song,
                    src,
                    feedback.clone(),
                );
                send_song(new_img, tx.clone(), tags).await;
            }
        }
    }

    Ok(())
}
