use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
    vec,
};

use bytes::Bytes;
use iced::{
    Element, Event, Subscription, Task, Theme, event, exit,
    keyboard::{Event::KeyReleased, Key, key::Named},
    widget::image::Handle,
    window::{self, change_icon, get_oldest, icon},
};
use rfd::{AsyncFileDialog, FileHandle};
use tokio::{runtime::RuntimeMetrics, sync::Semaphore, time::sleep};

use crate::{
    api::queue::{Queue, QueueMessage, ReturnSongImg, Source::Youtube, TagsInput},
    app::{
        song::{Song, SongId, SongState},
        song_img::{ImgFormat, ImgHash, SongImg},
        styles::*,
        view::{REGEX_LIM, view},
    },
    parser::{
        file_parser::{ImageSettings, ParseSettings, RegexType, apply_selected, get_tags_data},
        image_parser::{self, decode},
    },
};

#[derive(Debug, Clone)]
pub enum Message {
    FileOpenStart,
    PathOpenEnd(Option<Vec<FileHandle>>),
    FolderOpenStart,
    GotPath(Vec<FileHandle>),
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
    FromQueue(QueueMessage),
    ProcessedArt(ReturnSongImg),
    Offset(f32),
    ImgSelect(SongId, ImgHash),
    ImgPreview(SongId, usize),
    ImgMenuToggle(bool, SongId, ImgHash),
    Start,
    AfterStart,
    Exit,
    Print(String),
    None,
}

#[derive(Default)]
pub struct State {
    pub list_offset: f32,
    pub songs: Vec<Song>,
    pub ui_blocked: bool,
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
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                })))
                // .chain(Task::future(async {
                //     sleep(Duration::from_millis(300)).await;
                //     None
                // }))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))))
                .chain(Task::done(FromQueue(QueueMessage::GotArt(ReturnSongImg {
                    img: SongImg::new(
                        Handle::from_bytes(Bytes::from_static(include_bytes!("../../foo/6.jpg"))),
                        ImgFormat::Jpg,
                        Youtube,
                    ),
                    hash: self.state.songs[0].hash,
                    id: 0,
                }))));
            }
            Exit => return exit(),

            Print(s) => {
                dbg!(s);
            }
            GotPath(vec) => {
                for path in vec {
                    let res = get_tags_data(path.into(), &self.state.parse_settings);
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
            PathOpenEnd(data) => {
                self.state.ui_blocked = false;
                if let Some(vec) = data {
                    return Task::done(GotPath(vec));
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

            DownscaleInput(num) => {
                let res = str::parse::<i32>(&num);
                if let Ok(num) = res {
                    self.state.img_settings.downscale = i32::max(100, i32::min(10000, num));
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
                // dbg!(self.time.elapsed().as_millis(), "FromQueue start");
                use QueueMessage::*;
                match mes {
                    GotArt(output) => {
                        return Task::perform(decode(output, self.decode_sem.clone()), |res| {
                            ProcessedArt(res.unwrap())
                            // if let Ok(ok) = out {
                            //     ProcessedArt(ok)
                            // } else {
                            //     Print(format!("img was not decoded: {}", out.unwrap_err()))
                            // }
                        });
                    }
                }
            }
            ProcessedArt(output) => {
                let song = &mut self.state.songs[output.id];
                song.imgs.push(output.img);
                // let res =
                //     image_parser::push_and_group(&mut song.img_groups, &mut song.imgs, output.img);
                // let _ = res.inspect_err(|e| println!("img was not added: {e}"));

                dbg!(self.time.elapsed().as_millis());
                self.time = Instant::now();
            }
            Offset(offset) => {
                self.state.list_offset = offset;
            }
            None => {}
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
        view(self)
    }
    pub fn theme(&self) -> Theme {
        miasma_theme()
    }
}
