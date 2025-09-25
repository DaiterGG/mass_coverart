use std::{path::PathBuf, vec};

use iced::{
    Alignment, Color, Element, Event,
    Length::{Fill, FillPortion},
    Subscription, Task, Theme,
    alignment::Horizontal,
    event, exit,
    keyboard::{Event::KeyReleased, Key, key::Named},
    theme::{
        Palette,
        palette::{self, Danger, Extended, Pair, Primary, Secondary, Success},
    },
    widget::{
        self, column, container, row,
        scrollable::{Direction, Scrollbar},
    },
    window::{self, change_icon, get_oldest, icon},
};
use parser::file_parser::FileParser;
use rfd::{AsyncFileDialog, FileHandle};

use iced::widget::scrollable;

use crate::{
    api::queue::{self, Art, Queue, TagsInput},
    app::{
        song::SongId,
        song::{Song, SongState},
        styles::*,
    },
    parser::{
        self,
        file_parser::{ImageSettings, ParseSettings, RegexType},
    },
};
use iced::widget::{button, checkbox, stack, text, text_input};

const REGEX_LIM: usize = 7;

pub const TEXT_SIZE: f32 = 14.0;
pub const HEADER_SIZE: f32 = 17.0;
pub const INNER_TEXT_SIZE: f32 = 14.0;
pub const BTN_SIZE: f32 = 25.0;

#[derive(Default)]
pub struct State {
    pub list_offset: f32,
    pub songs: Vec<Song>,
    ui_blocked: bool,
    init_size: (f32, f32),
    img_settings: ImageSettings,
    parse_settings: ParseSettings,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileOpenStart,
    PathOpenEnd(Option<Vec<FileHandle>>),
    FolderOpenStart,
    FolderOpenEnd(Option<Vec<FileHandle>>),
    GotPath(Vec<FileHandle>),
    AddRegex,
    RemoveRegex,
    ParseToggle,
    RecursiveToggle,
    FilterPressed(usize),
    SeparatorPressed(usize, String),
    TitleInput(SongId, String),
    AlbumInput(SongId, String),
    ArtistInput(SongId, String),
    ConfirmSong(SongId),
    DiscardSong(SongId),
    Offset(f32),
    GotArt(Art),
    Start,
    Exit,
}

#[derive(Default)]
pub struct CoverUI {
    pub state: State,
}
impl CoverUI {
    pub fn init(init_size: (f32, f32)) -> (Self, Task<Message>) {
        let t = get_oldest()
            .and_then(move |id| {
                let icon =
                    icon::from_file_data(include_bytes!("../../resources/icon.png"), None).unwrap();
                change_icon::<Message>(id, icon)
            })
            .chain(Task::done(Message::Start));
        (
            Self {
                state: State {
                    init_size,
                    ..Default::default()
                },
            },
            t,
        )
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        // println!("{:?}", message);

        use Message::*;

        match message {
            Start => {
                #[cfg(debug_assertions)]
                return Task::done(GotPath(vec![
                    PathBuf::new().join("D:\\desk\\mass_coverart\\foo\\").into(),
                ]));
            }
            Exit => return exit(),
            GotPath(vec) => {
                for path in vec {
                    let res = FileParser::get_tags_data(path.into(), &self.state.parse_settings);
                    if let Err(e) = res {
                        println!("{e:?}");
                        return Task::none();
                    }
                    for i in res.unwrap() {
                        self.state.songs.push(Song::new(i));
                    }
                }
            }
            FileOpenStart => {
                self.state.ui_blocked = true;

                let files = AsyncFileDialog::new()
                    .add_filter("audio", &["mp3", "m4a"])
                    .add_filter("all", &["*"])
                    .set_directory("/")
                    .pick_files();
                return Task::perform(files, PathOpenEnd);
            }
            FolderOpenStart => {
                self.state.ui_blocked = true;
                let files = AsyncFileDialog::new().set_directory("/").pick_folders();
                return Task::perform(files, PathOpenEnd);
            }
            PathOpenEnd(data) => {
                self.state.ui_blocked = false;
                if let Some(vec) = data {
                    return Task::done(GotPath(vec));
                }
            }
            FilterPressed(i) => {
                self.state.parse_settings.reg_keys[i] =
                    self.state.parse_settings.reg_keys[i].next();
            }
            SeparatorPressed(i, sep) => {
                if i < self.state.parse_settings.reg_separators.len() {
                    self.state.parse_settings.reg_separators[i] = sep;
                }
            }
            AddRegex => {
                let st = &mut self.state.parse_settings;
                if st.reg_keys.len() < REGEX_LIM {
                    st.reg_keys.push(RegexType::Artist);
                    st.reg_separators.push(" - ".to_string());
                }
            }
            RemoveRegex => {
                let st = &mut self.state.parse_settings;
                if st.reg_keys.len() > 1 {
                    st.reg_keys.pop();
                    st.reg_separators.pop();
                }
            }
            RecursiveToggle => {
                self.state.parse_settings.recursive = !self.state.parse_settings.recursive;
            }
            ParseToggle => {
                self.state.parse_settings.parse_file_name =
                    !self.state.parse_settings.parse_file_name;
            }
            TitleInput(id, s) => self.state.songs[id].tag_data.title = Some(s),
            AlbumInput(id, s) => self.state.songs[id].tag_data.album = Some(s),
            ArtistInput(id, s) => self.state.songs[id].tag_data.artist = Some(s),
            ConfirmSong(id) => {
                let info = TagsInput::from_data(id, &self.state.songs[id].tag_data);
                self.state.songs[id].state = SongState::Main;
                return Queue::init(info);
            }

            DiscardSong(id) => {
                self.state.songs[id].state = SongState::Hidden;
                //Lazy GC
                while !self.state.songs.is_empty()
                    && self.state.songs.last().unwrap().state == SongState::Hidden
                {
                    self.state.songs.pop();
                }
            }
            GotArt(art) => {
                self.state.songs[art.id].imgs.push(art.img);
            }
            Offset(offset) => {
                self.state.list_offset = offset;
            }
            _ => unimplemented!("unhandled message"),
        }
        Task::none()
    }
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _windows| match event {
            Event::Window(window::Event::FileDropped(path)) => {
                Some(Message::GotPath(vec![path.into()]))
            }
            Event::Keyboard(KeyReleased {
                key: Key::Named(Named::Escape),
                ..
            }) => Some(Message::Exit),
            _ => None,
        })
    }
    pub fn view(&self) -> Element<'_, Message> {
        use Message::*;

        let theme = self.theme();
        println!("draw");

        let h2 = |s| {
            text(s)
                .size(TEXT_SIZE)
                .wrapping(text::Wrapping::None)
                .line_height(1.7)
        };
        let h3 = |s| {
            text(s)
                .size(INNER_TEXT_SIZE)
                .width(Fill)
                .height(Fill)
                .wrapping(text::Wrapping::None)
        };
        let btn = |s| button(h3(s).center()).clip(true).height(BTN_SIZE);
        let file_button = btn("file...")
            .width(50)
            .style(button_st)
            .on_press(FileOpenStart);
        let folder_row = row![
            btn("folder...")
                .width(70)
                .style(button_st)
                .on_press(FolderOpenStart),
            text("").width(10),
            checkbox("", self.state.parse_settings.recursive)
                .size(BTN_SIZE)
                .on_toggle(|_| RecursiveToggle)
                .style(check_st),
            h2("recursive"),
        ];
        let mut regex = row![];
        if self.state.parse_settings.parse_file_name {
            let set = &self.state.parse_settings;
            for i in 0..set.reg_keys.len() {
                let elem = Element::from(container(
                    btn(set.reg_keys[i].to_str())
                        .width(60)
                        .height(BTN_SIZE)
                        .style(button_st)
                        .on_press(FilterPressed(i)),
                ));
                regex = regex.push(elem);
                if i < set.reg_keys.len() - 1 {
                    let elem = Element::from(container(
                        text_input("", &set.reg_separators[i])
                            .style(input_st)
                            .width(30)
                            .align_x(Alignment::Center)
                            .size(INNER_TEXT_SIZE)
                            .on_input(move |s| SeparatorPressed(i, s)),
                    ));
                    regex = regex.push(elem);
                }
            }
            regex = regex.spacing(5).height(BTN_SIZE);

            let add = stack![
                text("").height(45).width(BTN_SIZE),
                button("")
                    .width(BTN_SIZE)
                    .height(BTN_SIZE)
                    .style(add_remove)
                    .on_press(AddRegex),
                text("˖")
                    .size(45)
                    .width(BTN_SIZE)
                    .height(45)
                    .align_x(Horizontal::Center)
                    .line_height(0.28)
                    .color(theme.extended_palette().secondary.base.color),
            ];
            let rem = stack![
                text("").height(45).width(BTN_SIZE),
                button("")
                    .width(BTN_SIZE)
                    .height(BTN_SIZE)
                    .style(add_remove)
                    .on_press(RemoveRegex),
                text("-")
                    .size(35)
                    .width(BTN_SIZE)
                    .height(45)
                    .align_x(Horizontal::Center)
                    .line_height(0.48)
                    .color(theme.extended_palette().secondary.base.color),
                text("-")
                    .size(65)
                    .width(BTN_SIZE)
                    .height(45)
                    .align_x(Horizontal::Center)
                    .line_height(0.32)
                    .color(theme.extended_palette().secondary.base.color),
            ];
            if self.state.parse_settings.reg_keys.len() > 1 {
                regex = regex.push(rem);
            }
            if self.state.parse_settings.reg_keys.len() < REGEX_LIM {
                regex = regex.push(add);
            }
        }
        let header_color = self.theme().extended_palette().background.weak.text;
        let files_panel = column![
            text("Open")
                .size(HEADER_SIZE)
                .width(Fill)
                .align_x(Alignment::Center)
                .color(header_color),
            file_button,
            folder_row,
            row![
                checkbox("", self.state.parse_settings.parse_file_name)
                    .on_toggle(|_| ParseToggle)
                    .size(BTN_SIZE)
                    .style(check_st),
                h2("parse file name"),
            ],
            regex.wrap(),
        ]
        .spacing(10);
        let settings_panel = column![
            text("Settings")
                .size(HEADER_SIZE)
                .width(Fill)
                .align_x(Alignment::Center)
                .color(header_color),
            text("").height(10),
            container(text("some").center()).width(Fill).height(50),
        ];

        let info_row = row![
            files_panel.height(Fill).width(FillPortion(1)),
            container(container("").style(bar_st).width(1).height(Fill))
                .width(30)
                .height(Fill)
                .padding(10),
            settings_panel.width(Fill).height(Fill)
        ];

        let list = Song::generate_view_list(self);
        let list = scrollable(list)
            .direction(Direction::Vertical(
                Scrollbar::new().margin(0).scroller_width(15),
            ))
            .width(Fill)
            .height(Fill)
            .spacing(0)
            .on_scroll(|v| Offset(v.relative_offset().y))
            .style(list_scroll_st);
        let drag_info = if self.state.songs.is_empty() {
            text("Drag and drop")
                .center()
                .size(50)
                .width(Fill)
                .height(Fill)
        } else {
            text("")
        };
        let list = stack![
            container(drag_info)
                .width(Fill)
                .height(Fill)
                .style(list_bg_st),
            row![list, text("").width(4)],
            container("").width(Fill).height(Fill).style(list_border_st)
        ];

        let main_col = column![
            info_row.height(FillPortion(4)).width(Fill),
            container(list).height(FillPortion(6)).width(Fill),
        ]
        .height(Fill)
        .width(Fill)
        .padding(15);

        if self.state.ui_blocked {
            container(
                text("Choose Items")
                    .center()
                    .size(50)
                    .height(Fill)
                    .width(Fill),
            )
            .height(Fill)
            .width(Fill)
            .style(filler_st)
            .into()
        } else {
            main_col.into()
        }
    }
    pub fn theme(&self) -> Theme {
        let primary = Color::from_rgb8(120, 130, 74);
        let secondary = Color::from_rgb8(187, 119, 68);
        let bg = Color::from_rgb8(34, 34, 34);
        let bg_strong = Color::from_rgb8(28, 28, 28);
        let bg_weak = Color::from_rgb8(46, 46, 46);
        let bg_text = Color::from_rgb8(90, 90, 90);
        let text = Color::from_rgb8(215, 196, 131);
        let success = Color::from_rgb8(95, 135, 95);
        let danger = Color::from_rgb8(104, 87, 66);
        Theme::custom_with_fn(
            "custom".to_string(),
            Palette {
                text,
                primary,
                success,
                danger,
                background: bg,
            },
            |_| Extended {
                primary: Primary::generate(primary, bg, text),
                background: palette::Background {
                    weak: Pair {
                        color: bg_weak,
                        text: bg_text,
                    },
                    base: Pair::new(bg, text),
                    strong: Pair::new(bg_strong, text),
                },
                secondary: Secondary::generate(secondary, secondary),
                success: Success::generate(success, bg, text),
                danger: Danger::generate(danger, bg, text),
                is_dark: true,
            },
        )
    }
}
