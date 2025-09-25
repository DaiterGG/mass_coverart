use anyhow::bail;
use iced::{
    futures::channel::mpsc::{Sender, TrySendError},
    widget::image::Handle,
};
use reqwest::Client;
use tokio::task::JoinSet;
use yt_search::{SearchFilters, YouTubeSearch};

use crate::{
    api::queue::{Art, ImgData, ImgFormat::Jpeg, TagsInput},
    app::{iced_app::Message, song::SongId},
};

pub async fn youtube(t: TagsInput, tx: Sender<Message>) -> Result<(), anyhow::Error> {
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

    let mut res = Vec::with_capacity(5);
    match search.search("rust programming", filters).await {
        Ok(results) => {
            for i in 0..usize::min(results.len(), 6) {
                dbg!(&results[i].video_id);
                res.push(results[i].video_id.clone());
            }
        }
        Err(e) => bail!("Search error: {}", e),
    }
    let client = Client::new();
    let mut set = JoinSet::new();
    for link_id in res {
        set.spawn(get_img(client.clone(), t.id, link_id, tx.clone()));
    }
    set.join_all().await;

    Ok(())
}
async fn get_img(client: Client, id: SongId, link_id: String, mut tx: Sender<Message>) {
    let req = client
        .get(format!(
            "https://img.youtube.com/vi/{link_id}/mqdefault.jpg"
        ))
        .build()
        .unwrap();
    let pic = client.execute(req).await.unwrap().bytes().await.unwrap();

    let mut tried = tx.try_send(Message::GotArt(Art {
        id,
        img: ImgData::Bytes(Handle::from_bytes(pic), Jpeg),
    }));
    loop {
        if tried.is_ok() {
            return;
        }
        let e = tried.err().unwrap();
        if e.is_full() {
            tried = tx.try_send(e.into_inner())
        } else {
            return;
        }
    }
}
