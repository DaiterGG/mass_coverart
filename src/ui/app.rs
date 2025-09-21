use std::{default, path::PathBuf};

use iced::{
    Element, Event,
    Length::{Fill, FillPortion},
    Subscription, Task, Theme, event, exit,
    keyboard::{Event::KeyReleased, Key, key::Named},
    widget::{Column, button, center, column, container, row, text},
    window,
};
use parser::file_parser::FileParser;

use crate::parser::{self, file_parser::TagData};
#[derive(Default)]
struct State {
    init_size: (f32, f32),
    settings: Settings,
    songs: Vec<Song>,
}
struct Song {
    tag_data: TagData,
}
impl Song {
    fn new(tag_data: TagData) -> Self {
        Self { tag_data }
    }
}

struct Settings {
    convert_to: EndFormat,
    downscale: (i32, i32),
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            convert_to: EndFormat::Jpeg,
            downscale: (1200, 1200),
        }
    }
}
enum EndFormat {
    Jpeg,
    Png,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileDropped(PathBuf),
    Exit,
}

#[derive(Default)]
pub struct CoverUI {
    state: State,
}
impl CoverUI {
    pub fn init(init_size: (f32, f32)) -> (Self, Task<Message>) {
        (
            Self {
                state: State {
                    init_size,
                    ..Default::default()
                },
            },
            Task::none(),
        )
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        println!("{:?}", message);
        match message {
            Message::FileDropped(path) => {
                let res = FileParser::get_tags_data(path);
                if res.is_err() {
                    dbg!("display error");
                    return Task::none();
                }
                for i in res.unwrap() {
                    self.state.songs.push(Song::new(i));
                }
            }
            Message::Exit => return exit(),
            _ => todo!("unhandled message"),
        }
        Task::none()
    }
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _windows| match event {
            Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
            Event::Keyboard(KeyReleased {
                key: Key::Named(Named::Escape),
                ..
            }) => Some(Message::Exit),
            _ => None,
        })
    }
    pub fn view(&self) -> Element<'_, Message> {
        let files_panel = column![
            text("Open (or drag and drop)").size(30),
            container(text("some").center()).width(Fill).height(50),
            container(text("some").center()).width(Fill).height(50),
            container(text("some").center()).width(Fill).height(50),
        ];
        let settings_panel = column![
            text("Settings").size(30),
            container(text("some").center()).width(Fill).height(50),
            container(text("some").center()).width(Fill).height(50),
            container(text("some").center()).width(Fill).height(50),
        ];

        let info_row = row![
            files_panel.height(Fill).width(FillPortion(1)),
            settings_panel.width(Fill).height(Fill)
        ];
        let list = column![
            container(text("some").center()).width(Fill).height(50),
            container(text("some").center()).width(Fill).height(50),
            container(text("some").center()).width(Fill).height(50),
        ]
        .width(Fill)
        .height(Fill);
        let list_area = container(list).style(container::rounded_box);
        let main_col = column![
            info_row.height(FillPortion(1)).width(Fill),
            list_area.height(Fill).width(Fill),
        ]
        .height(Fill)
        .padding(15);
        container(main_col).width(Fill).height(Fill).into()
    }
    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}
