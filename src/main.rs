#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod app;
mod parser;

use app::iced_app::CoverUI;
use iced::Size;

pub type ImgHandle = iced::widget::image::Handle;
pub type TaskHandle = iced::task::Handle;

// #[tokio::main]
// async fn main() {
// }
fn main() -> iced::Result {
    let init_size = (800.0, 600.0);
    iced::application("Mass CoverArt", CoverUI::update, CoverUI::view)
        .window_size(Size::new(init_size.0, init_size.1))
        .theme(CoverUI::theme)
        .subscription(CoverUI::subscription)
        .centered()
        .run_with(move || CoverUI::init(init_size))
    // .run()

    // let pic_data: Vec<u8> = read(path).unwrap();
    // let req = reqwest::get("https://img.youtube.com/vi/4PDoT7jtxmw/mqdefault.jpg")
    //     .await
    //     .unwrap();
    // let pic = req.bytes().await.unwrap();
    // let pic = Picture {
    //     mime_type: MimeType::Jpeg,
    //     data: &pic,
    // };
    // tag.set_album_cover(pic);
    // tag.write_to_path("./foo/Wierd Al - Hardware Store.m4a")
    //     .unwrap();
}
