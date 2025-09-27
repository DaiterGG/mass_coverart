use std::time::{Duration, Instant};

use anyhow::{Error, bail};
use iced::{
    futures::channel::mpsc::{Sender, TrySendError},
    widget::{image::Handle, shader::wgpu::core::present},
};
use reqwest::Client;
use tokio::{task::JoinSet, time::sleep};
use yt_search::{SearchFilters, YouTubeSearch};

use crate::{
    api::queue::{ReturnSongImg, Source::Youtube, TagsInput},
    app::{
        iced_app::Message,
        song::SongId,
        song_img::{ImgFormat::Jpg, SongImg},
    },
};

const SEARCH_LIMIT: usize = 20;
pub async fn youtube(tags: TagsInput, tx: Sender<Message>) -> Result<(), Error> {
    if tags.album.is_none() | tags.artist.is_none() {
        bail!("Provider not suitable")
    }
    let prompt = format!("{} {} album", tags.artist.unwrap(), tags.album.unwrap());
    dbg!(&prompt);

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

    let mut res = Vec::with_capacity(SEARCH_LIMIT);
    match search.search(&prompt, filters).await {
        Ok(results) => {
            let mut limit = SEARCH_LIMIT;
            let mut i = 0;
            dbg!(&results.len());
            while i < limit && i < results.len() {
                if !results[i].duration.starts_with('0')
                    && !results[i].duration.eq_ignore_ascii_case("1:00")
                {
                    dbg!(&results[i].video_id);
                    res.push(results[i].video_id.clone());
                } else {
                    limit += 1;
                }
                i += 1;
            }
        }
        Err(e) => bail!("Search error: {}", e),
    }
    let client = Client::new();
    let mut set = JoinSet::new();
    for link_id in res {
        set.spawn(get_img(
            client.clone(),
            tags.id,
            tags.hash,
            link_id,
            tx.clone(),
        ));
    }
    let results = set.join_all().await;
    for r in results {
        let _ = r.inspect_err(|e| {
            dbg!(e);
        });
    }

    Ok(())
}
async fn get_img(
    client: Client,
    id: SongId,
    hash: u64,
    link_id: String,
    mut tx: Sender<Message>,
) -> Result<(), Error> {
    let req = client
        .get(format!(
            "https://img.youtube.com/vi/{link_id}/maxresdefault.jpg"
        ))
        .build()?;
    let mut res = client.execute(req).await;
    if let Err(e) = res {
        dbg!(e);
        let req = client
            .get(format!("https://img.youtube.com/vi/{link_id}/hq720.jpg"))
            .build()?;
        res = client.execute(req).await;
        if let Err(e) = res {
            dbg!(e);
            let req = client
                .get(format!(
                    "https://img.youtube.com/vi/{link_id}/sddefault.jpg"
                ))
                .build()?;
            res = client.execute(req).await;
            if let Err(e) = res {
                return Err(e.into());
            }
        }
    }

    let pic = res?.bytes().await?;
    if pic.len() == 1097 {
        bail!("\"No image\" received");
    }

    let mut tried = tx.try_send(Message::FromQueue(crate::api::queue::QueueMessage::GotArt(
        ReturnSongImg::new(
            id,
            hash,
            SongImg::new(Handle::from_bytes(pic), Jpg, Youtube),
        ),
    )));
    loop {
        if let Err(e) = tried
            && e.is_full()
        {
            tried = tx.try_send(e.into_inner())
        } else {
            break;
        }
    }
    Ok(())
}
