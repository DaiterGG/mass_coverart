#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod app;
mod parser;

use app::iced_app::CoverUI;
use flexi_logger::{Duplicate::Info, FileSpec, Logger, WriteMode};
use iced::Size;
use log::info;
use musicbrainz_rs::{
    Browse, Fetch, FetchCoverart, FetchCoverartQuery, MusicBrainzClient, Search,
    entity::{
        CoverartResponse,
        artist::{Artist, ArtistSearchQuery},
        release::{self, Release, ReleaseSearchQuery, ReleaseSearchQueryLuceneQueryBuilder},
    },
};

pub type ImgHandle = iced::widget::image::Handle;
pub type TaskHandle = iced::task::Handle;

// #[tokio::main]
// async fn main() -> Result<(), anyhow::Error> {
//     Ok(())
// }
fn main() -> Result<(), anyhow::Error> {
    Logger::try_with_str("info")?
        .log_to_file(
            FileSpec::default()
                .basename("mass_coverart")
                .use_timestamp(false),
        )
        .duplicate_to_stdout(Info)
        .print_message()
        .start()?;

    let init_size = (800.0, 600.0);
    iced::application("Mass CoverArt", CoverUI::update, CoverUI::view)
        .window_size(Size::new(init_size.0, init_size.1))
        .theme(CoverUI::theme)
        .subscription(CoverUI::subscription)
        .centered()
        .run_with(move || CoverUI::init(init_size))?;
    Ok(())
}
