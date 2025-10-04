#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod app;
mod parser;

use std::io::Cursor;

use app::iced_app::CoverUI;
use flexi_logger::{Duplicate::Info, FileSpec, LogSpecification, Logger};
use iced::Size;
use image::{ImageFormat, ImageReader, codecs::jpeg::JpegDecoder};
use reqwest::Client;
use serde::Deserialize;
pub type ImgHandle = iced::widget::image::Handle;
pub type TaskHandle = iced::task::Handle;

#[derive(Deserialize)]
struct PossibleRedirect {
    pub url: String,
}
fn main() -> Result<(), anyhow::Error> {
    Logger::with(LogSpecification::info())
        .log_to_file(
            FileSpec::default()
                .basename("mass_coverart")
                .use_timestamp(false),
        )
        .duplicate_to_stdout(Info)
        .print_message()
        .start()?;

    let init_size = (800.0, 600.0);
    iced::application(
        move || CoverUI::init(init_size),
        CoverUI::update,
        CoverUI::view,
    )
    .title("Mass CoverArt")
    .window_size(Size::new(init_size.0, init_size.1))
    .theme(CoverUI::theme)
    .subscription(CoverUI::subscription)
    .centered()
    .run()?;
    Ok(())
}
