use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
    vec,
};

use bytes::Bytes;
use iced::{
    Element, Event, Subscription, Task, Theme, event, exit,
    keyboard::{Event::KeyReleased, Key, key::Named},
    stream,
    window::{self, change_icon, get_oldest, icon},
};
use rfd::{AsyncFileDialog, FileHandle};
use tokio::{sync::Semaphore, task::yield_now, time::sleep};

use crate::{
    ImgHandle,
    api::queue::{Queue, QueueMessage, ReturnSongImg, Source::Youtube, TagsInput},
    app::{
        song::{Song, SongHash, SongId, SongState},
        song_img::{ImgFormat, ImgHash, ImgId, SongImg},
        styles::*,
        view::{REGEX_LIM, view},
    },
    parser::{
        file_parser::{ParseSettings, RegexType, TagData, apply_selected, get_tags_data},
        image_parser::{self, ImageSettings, decode_and_sample},
    },
};

#[derive(Debug, Clone)]
pub enum Message {
    FileOpenStart,
    PathOpenEnd(Option<Vec<FileHandle>>),
    FolderOpenStart,
    GotPath(Vec<FileHandle>),
    CreateSongs(Vec<TagData>),
    PushSong(Song),
    PathDropped(Vec<FileHandle>),
    DownscaleInput(String),
    AddRegex,
    RemoveRegex,
    ParseToggle,
    SquareToggle,
    JpgToggle,
    RecursiveToggle,
    FilterPressed(usize),
    SeparatorInput(usize, String),
    TitleInput(SongId, String),
    AlbumInput(SongId, String),
    ArtistInput(SongId, String),
    ConfirmSong(SongId),
    DiscardSong(SongId),
    GoBackDiscard(SongId),
    GoBack(SongId),
    Accept(SongId),
    // INFO: potentially return imgs into mtx
    FromQueue(QueueMessage),
    ProcessedArt(ReturnSongImg),
    Offset(f32),
    ImgSelect(SongId, ImgHash),
    ImgPreview(SongId, ImgId),
    ImgPreviewClose,
    ImgMenuToggle(bool, SongId, ImgHash),
    Start,
    AfterStart,
    Exit,
    Print(String),
}

#[derive(Default)]
pub struct State {
    pub list_offset: f32,
    pub songs: Vec<Song>,
    pub preview_img: Option<ImgHandle>,
    pub ui_blocked: bool,
    pub ui_loading: bool,
    pub parse_settings: ParseSettings,
    pub init_size: (f32, f32),
    pub img_settings: ImageSettings,
}
pub struct CoverUI {
    pub time: Instant,
    pub state: State,
    pub decode_sem: Arc<Semaphore>,
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
                decode_sem: Arc::new(Semaphore::new(1)),
                time: Instant::now(),
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
                ]))
                .chain(Task::done(AfterStart));
            }
            AfterStart => {
                return Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Bytes::from_static(include_bytes!("../../foo/2.jpg")),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: 0,
                    id: 0,
                })))
                .chain(Task::future(async {
                    sleep(Duration::from_millis(3000)).await;
                    Print("".to_string())
                }))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Bytes::from_static(include_bytes!("../../foo/2.jpg")),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: 0,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Bytes::from_static(include_bytes!("../../foo/2.jpg")),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: 0,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Bytes::from_static(include_bytes!("../../foo/2.jpg")),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: 0,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Bytes::from_static(include_bytes!("../../foo/2.jpg")),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: 0,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Bytes::from_static(include_bytes!("../../foo/2.jpg")),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: 0,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Bytes::from_static(include_bytes!("../../foo/2.jpg")),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: 0,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Bytes::from_static(include_bytes!("../../foo/2.jpg")),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: 0,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Bytes::from_static(include_bytes!("../../foo/2.jpg")),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: 0,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Bytes::from_static(include_bytes!("../../foo/6.jpg")),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: 0,
                    id: 0,
                }))));
            }
            Exit => return exit(),

            Print(s) => {
                dbg!(s);
            }
            GotPath(vec) => {
                self.state.ui_loading = false;
                return Task::perform(
                    get_tags_data(vec, self.state.parse_settings.clone()),
                    |res| {
                        if let Err(e) = res {
                            return Print(e.to_string());
                        }
                        CreateSongs(res.unwrap())
                    },
                );
            }
            CreateSongs(tags) => {
                return Task::stream(stream::channel(1, |mut tx| async move {
                    for tag in tags {
                        let mut tried = tx.try_send(PushSong(Song::new(tag)));
                        loop {
                            if let Err(e) = tried
                                && e.is_full()
                            {
                                dbg!("full");
                                tried = tx.try_send(e.into_inner())
                            } else {
                                break;
                            }
                        }
                        yield_now().await;
                    }
                }));
            }
            PushSong(song) => {
                self.state.songs.push(song);
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
                self.state.ui_loading = true;
                if let Some(vec) = data {
                    return Task::done(GotPath(vec));
                }
            }
            PathDropped(vec) => {
                self.state.ui_loading = true;
                return Task::done(GotPath(vec));
            }

            ImgSelect(song_id, img_hash) => {
                self.state.songs[song_id].selected_img = img_hash;
                return Task::done(ImgMenuToggle(false, song_id, img_hash));
            }
            ImgMenuToggle(enter, song_id, img_hash) => {
                if enter {
                    self.state.songs[song_id].menu_img = img_hash;
                } else if self.state.songs[song_id].menu_img == img_hash {
                    self.state.songs[song_id].menu_close();
                }
            }
            SquareToggle => {
                self.state.img_settings.square = !self.state.img_settings.square;
            }
            FilterPressed(i) => {
                self.state.parse_settings.reg_keys[i] =
                    self.state.parse_settings.reg_keys[i].next();
            }
            SeparatorInput(i, sep) => {
                if i < self.state.parse_settings.reg_separators.len() {
                    self.state.parse_settings.reg_separators[i] = sep;
                }
            }
            ImgPreview(song_i, img_id) => {
                let img = self.state.songs[song_i].imgs[img_id]
                    .get_final_preview(&self.state.img_settings);

                self.state.preview_img = Some(img);
            }
            ImgPreviewClose => {
                self.state.preview_img = None;
            }
            DownscaleInput(num) => {
                let res = str::parse::<u32>(&num);
                if let Ok(num) = res {
                    self.state.img_settings.downscale = u32::min(10000, num);
                } else {
                    self.state.img_settings.downscale = 0;
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
                let info = TagsInput::from_data(
                    id,
                    self.state.songs[id].hash,
                    &self.state.songs[id].tag_data,
                );
                self.state.songs[id].state = SongState::Main;
                let (q, handle) = Queue::init(info);
                self.state.songs[id].queue_handle = Some(handle);
                return q;
            }
            GoBackDiscard(id) => return Task::done(GoBack(id)).chain(Task::done(DiscardSong(id))),
            Accept(id) => {
                apply_selected(self, id);
                return Task::done(GoBack(id));
            }
            GoBack(id) => {
                self.state.songs[id].state = SongState::Confirm;

                self.state.songs[id].queue_handle.take().unwrap().abort();
                self.state.songs[id].imgs.clear();
                self.state.songs[id].img_groups.clear();
                self.state.songs[id].menu_close();
                self.state.songs[id].unselect();
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
            JpgToggle => {
                self.state.img_settings.jpg = !self.state.img_settings.jpg;
            }

            FromQueue(mes) => {
                use QueueMessage::*;
                match mes {
                    GotArt(output) => {
                        // song was deleted
                        if output.id >= self.state.songs.len()
                        // FIXME:
                        // || self.state.songs[output.id].hash != output.hash
                        {
                            return Task::done(Print("skipped".to_string()));
                        }
                        return Task::perform(
                            decode_and_sample(
                                output,
                                self.decode_sem.clone(),
                                self.state.img_settings,
                            ),
                            |res| {
                                if let Ok(ok) = res {
                                    ProcessedArt(ok)
                                } else {
                                    Print(format!("img was not decoded: {}", res.unwrap_err()))
                                }
                            },
                        );
                    }
                }
            }
            ProcessedArt(output) => {
                let song = &mut self.state.songs[output.id];
                // song.imgs.push(output.img);
                let res =
                    image_parser::push_and_group(&mut song.img_groups, &mut song.imgs, output.img);

                let _ = res.inspect_err(|e| println!("img was not added: {e}"));
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
                Some(Message::PathDropped(vec![path.into()]))
            }
            Event::Keyboard(KeyReleased {
                key: Key::Named(Named::Escape),
                ..
            }) => Some(Message::Exit),
            _ => None,
        })
    }
    pub fn view(&self) -> Element<'_, Message> {
        view(self)
    }
    pub fn theme(&self) -> Theme {
        miasma_theme()
    }
}
