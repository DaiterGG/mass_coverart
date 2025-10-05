use core::{hash, panic};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    vec,
};

use bytes::Bytes;
use iced::{
    Element, Event,
    Length::Fill,
    Subscription, Task, Theme, event, exit,
    futures::channel::mpsc::Sender,
    keyboard::{Event::KeyReleased, Key, Modifiers, key::Named},
    stream,
    widget::{Column, Space, button, column, container, image, scrollable, stack},
    window::{self, icon},
};
use log::{error, info, trace, warn};
use reqwest::Client;
use rfd::{AsyncFileDialog, FileHandle};
use tokio::{sync::Semaphore, task::yield_now, time::sleep};

use crate::{
    api::{
        queue::{Queue, QueueMessage, Source::YoutubeAlbum, TagsInput},
        shared,
    },
    app::{
        song::{Song, SongHash, SongId, SongState},
        song_img::{ImageProgress, ImageSettings, ImgFormat, ImgHash, ImgId, SongImg},
        styles::*,
        view::{PreviewState, REGEX_LIM, view},
    },
    parser::file_parser::{ParseSettings, RegexType, TagData, apply_selected, get_tags_data},
};
#[derive(Clone)]
pub enum Message {
    FileOpenStart,
    PathOpenEnd(Option<Vec<FileHandle>>),
    FolderOpenStart,
    GotPath(Vec<FileHandle>),
    CreateSongs(Vec<TagData>),
    PushSongs(Vec<Song>),
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
    ConfirmSongIfNot(SongId),
    AutoModToggle(bool),
    DiscardSong(SongId),
    GoBackDiscard(SongId),
    GoBack(SongId),
    ApplySelectedPressed(SongId),
    ApplySelected(SongId),
    DecodeAccept(Bytes, ImgFormat, SongId),
    AcceptFailed(SongId),

    // INFO: potentially return imgs into mtx
    FromQueue(SongId, SongHash, QueueMessage),
    ConfirmLastSong,
    ProcessedArt(SongId, SongHash, SongImg),
    Scroll(f32),
    ImgSelect(SongId, ImgHash),
    ImgPreviewOpen(SongId, ImgId),
    ImgPreview(SongId, ImgId),
    ImgPreviewSet(PreviewState),
    DecodePreview(Bytes, ImgFormat, SongId, ImgId),
    ImgMenuToggle(bool, SongId, ImgHash),
    Start,
    AfterStart,
    Exit,
    Nothing,
}

#[derive(Default)]
pub struct State {
    pub list_scroll: f32,
    pub songs: Vec<Song>,
    pub preview_img: PreviewState,
    pub ui_blocked: bool,
    pub ui_loading: bool,
    pub parse_settings: ParseSettings,
    pub init_size: (f32, f32),
    pub auto_mod: bool,
    pub auto_mod_current_song: usize,
    pub img_settings: ImageSettings,
}
pub fn song_is_invalid(st: &State, id: SongId, hash: SongHash) -> bool {
    if id >= st.songs.len()
    // FIXME:
    // || self.state.songs[output.id].hash != output.hash
    {
        error!("attempt to access invalid song {}", id);
        true
    } else {
        false
    }
}
pub struct CoverUI {
    pub time: Instant,
    pub state: State,
    pub decode_sem: Arc<Semaphore>,
    pub theme: Option<Theme>,
}
impl CoverUI {
    pub fn init(init_size: (f32, f32)) -> (Self, Task<Message>) {
        let t = window::oldest()
            .and_then(move |id| {
                let icon =
                    icon::from_file_data(include_bytes!("../../resources/icon.png"), None).unwrap();
                window::set_icon(id, icon)
            })
            .chain(Task::done(Message::Start));
        (
            Self {
                theme: Some(miasma_theme()),
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
                #[cfg(debug_assertions)]
                return Task::future(async {
                    sleep(Duration::from_millis(3000)).await;
                    Nothing
                })
                .chain(Task::done(FromQueue(
                            0,
                            0,
                        QueueMessage::GotArt(SongImg::new(
                         ImgFormat::Jpg,
                         ImageProgress::Raw(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        YoutubeAlbum,
                        "AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA ".to_string()
                        ))))
                )
                .chain(Task::future(async {
                    ConfirmSongIfNot(0)
                }));
            }
            Nothing => {}
            GotPath(vec) => {
                self.state.ui_loading = false;

                return Task::perform(
                    get_tags_data(vec, self.state.parse_settings.clone()),
                    |res| {
                        if let Err(e) = res {
                            warn!("{e}");
                            return Nothing;
                        }
                        PushSongs(res.unwrap())
                    },
                );
            }
            PushSongs(songs) => {
                self.state.songs.extend(songs);
                info!("{} songs now", self.state.songs.len());
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
                    self.state.ui_loading = true;
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
                    let now = self.state.songs[song_id].menu_img;
                    if now != img_hash {
                        self.state.songs[song_id].menu_img = img_hash;
                    }
                } else {
                    let now = self.state.songs[song_id].menu_img;
                    if now == img_hash {
                        self.state.songs[song_id].menu_close();
                    }
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
            ImgPreviewOpen(song_i, img_id) => {
                if let ImageProgress::Preview(urls) = &self.state.songs[song_i].imgs[img_id].image {
                    let format = ImgFormat::from_url(urls.first().unwrap());
                    let (t, h) =
                        Task::perform(shared::get_img(Client::new(), urls.to_vec()), move |res| {
                            if let Ok(bytes) = res {
                                DecodePreview(bytes, format, song_i, img_id)
                            } else {
                                ImgPreviewSet(PreviewState::Error)
                            }
                        })
                        .abortable();
                    self.state.preview_img = PreviewState::Downloading(h);
                    return t;
                } else {
                    self.state.preview_img = PreviewState::Loading;
                    return Task::done(ImgPreview(song_i, img_id));
                }
            }

            DecodePreview(bytes, format, song_id, img_id) => {
                self.state.preview_img = PreviewState::Loading;
                let res = self.state.songs[song_id].imgs[img_id].preview_to_decoded(bytes, format);

                if let Err(e) = res {
                    error!(
                        "preview img was not decoded: {e}, format: {format:?}, feedback: {} ",
                        self.state.songs[song_id].imgs[img_id].feedback
                    );
                    return Task::none();
                }
                return Task::done(ImgPreview(song_id, img_id));
            }
            ImgPreview(song_i, img_id) => {
                let state = self.state.songs[song_i].imgs[img_id]
                    .final_img_preview(self.state.img_settings);
                self.state.preview_img = PreviewState::Display(state);
            }
            ImgPreviewSet(state) => {
                if let PreviewState::Downloading(h) = &self.state.preview_img
                    && let PreviewState::Closed = state
                {
                    h.abort()
                }
                self.state.preview_img = state;
            }
            ApplySelectedPressed(song_id) => {
                let song = &mut self.state.songs[song_id];
                let selected_img_hash = song.selected_img;
                let mut selected_img_id = None;
                for i in 0..song.imgs.len() {
                    if song.imgs[i].hash == selected_img_hash {
                        selected_img_id = Some(i);
                        break;
                    }
                }
                if let Some(img_id) = selected_img_id {
                    let img = &mut song.imgs[img_id];

                    info!("Apply Select final img {}", img.image.dbg());
                    if let ImageProgress::Preview(urls) = &img.image {
                        song.state = SongState::MainDownloading;
                        let format = ImgFormat::from_url(urls.first().unwrap());
                        return Task::perform(
                            shared::get_img(Client::new(), urls.to_vec()),
                            move |res| {
                                if let Ok(bytes) = res {
                                    DecodeAccept(bytes, format, song_id)
                                } else {
                                    error!("{}", res.unwrap_err());
                                    AcceptFailed(song_id)
                                }
                            },
                        );
                    } else {
                        song.state = SongState::MainLoading;
                        return Task::done(ApplySelected(song_id));
                    }
                }
            }

            AcceptFailed(song_id) => {
                self.state.songs[song_id].state = SongState::Main;
            }
            DecodeAccept(bytes, format, song_id) => {
                let song = &mut self.state.songs[song_id];
                let selected_img_hash = song.selected_img;
                let mut selected_img_id = None;
                for i in 0..song.imgs.len() {
                    if song.imgs[i].hash == selected_img_hash {
                        selected_img_id = Some(i);
                        break;
                    }
                }
                if let Some(img_id) = selected_img_id {
                    self.state.songs[song_id].state = SongState::MainLoading;
                    let res =
                        self.state.songs[song_id].imgs[img_id].preview_to_decoded(bytes, format);

                    info!(
                        "DecodeAccept final img {} {}",
                        self.state.songs[song_id].imgs[img_id].image.dbg(),
                        self.state.songs[song_id].imgs[img_id].hash
                    );
                    if let Err(e) = res {
                        error!(
                            "accepted img was not decoded: {e}, format: {format:?}, feedback: {} ",
                            self.state.songs[song_id].imgs[img_id].feedback
                        );
                        self.state.songs[song_id].state = SongState::Main;
                        return Task::none();
                    }

                    return Task::done(ApplySelected(song_id));
                }
            }
            ApplySelected(song_id) => {
                if let Err(e) = apply_selected(self, song_id) {
                    error!("{}", e);
                    return Task::done(DiscardSong(song_id));
                }
                return Task::done(GoBack(song_id));
            }
            ConfirmSongIfNot(id) => {
                if self.state.songs.len() > id && self.state.songs[id].state == SongState::Confirm {
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
            }
            GoBackDiscard(id) => return Task::done(GoBack(id)).chain(Task::done(DiscardSong(id))),

            GoBack(id) => {
                self.state.songs[id].state = SongState::Confirm;

                self.state.songs[id].queue_handle.take().unwrap().abort();
                self.state.songs[id].imgs.clear();
                self.state.songs[id].img_groups.clear();
                self.state.songs[id].menu_close();
                self.state.songs[id].unselect();
                if self.state.auto_mod {
                    return Task::done(ConfirmLastSong);
                }
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
            TitleInput(id, s) => {
                self.state.songs[id].tag_data.title = if s.is_empty() { None } else { Some(s) }
            }
            AlbumInput(id, s) => {
                self.state.songs[id].tag_data.album = if s.is_empty() { None } else { Some(s) }
            }
            ArtistInput(id, s) => {
                self.state.songs[id].tag_data.artist = if s.is_empty() { None } else { Some(s) }
            }

            JpgToggle => {
                self.state.img_settings.jpg = !self.state.img_settings.jpg;
            }

            FromQueue(id, hash, mes) => {
                if song_is_invalid(&self.state, id, hash) {
                    // song was deleted
                    return Task::none();
                }
                use QueueMessage::*;
                match mes {
                    GotArt(output) => {
                        return Task::perform(
                            SongImg::decode_and_sample(output, self.decode_sem.clone()),
                            move |res| {
                                if let Ok(ok) = res {
                                    ProcessedArt(id, hash, ok)
                                } else {
                                    error!(
                                        "img was not decoded and sapmled: {},",
                                        res.unwrap_err()
                                    );
                                    Nothing
                                }
                            },
                        );
                    }
                    SetSources(num, out_of) => {
                        self.state.songs[id].sources_finished = (num, out_of);
                        if self.state.auto_mod && num == out_of {
                            let mut task = Task::none();
                            if self.state.songs[id].state == SongState::Main
                                && !self.state.songs[id].imgs.is_empty()
                            {
                                let hash = self.state.songs[id].imgs[0].hash;
                                task = task.chain(Task::done(ImgSelect(id, hash)));
                            }
                            return task.chain(Task::done(ConfirmLastSong));
                        }
                    }
                    SourceFinished => {
                        let now = self.state.songs[id].sources_finished.0;
                        self.state.songs[id].sources_finished = (now + 1, Queue::TOTAL_SOURCES)
                    }
                }
            }
            ConfirmLastSong => {
                let cur = self.state.auto_mod_current_song;
                // current song in process right now
                if self.state.songs.len() > cur {
                    let song = &mut self.state.songs[cur];
                    let (a, b) = song.sources_finished;
                    if song.state != SongState::Confirm && a != b {
                        return Task::none();
                    }
                }
                for i in (0..self.state.songs.len()).rev() {
                    if self.state.songs[i].state != SongState::Confirm {
                        continue;
                    }
                    self.state.auto_mod_current_song = i;
                    return Task::done(ConfirmSongIfNot(i));
                }
            }
            ProcessedArt(id, hash, output) => {
                if song_is_invalid(&self.state, id, hash) {
                    return Task::none();
                }
                let song = &mut self.state.songs[id];
                let res = output.push_and_group(&mut song.img_groups, &mut song.imgs);

                let _ = res.inspect_err(|e| warn!("img was not added: {e}"));
            }
            Scroll(scroll_uv) => {
                self.state.list_scroll = scroll_uv;
            }
            AutoModToggle(on) => {
                self.state.auto_mod = on;
                if on {
                    return Task::done(ConfirmLastSong);
                }
            }
            _ => unimplemented!("unhandled message"),
        }
        Task::none()
    }
    pub fn subscription(&self) -> Subscription<Task<Message>> {
        event::listen_with(|event, _status, _windows| match event {
            Event::Window(window::Event::FileDropped(path)) => {
                Some(Task::done(Message::PathDropped(vec![path.into()])))
            }
            Event::Keyboard(KeyReleased {
                #[cfg(debug_assertions)]
                    key: Key::Named(Named::Escape),
                ..
            })
            | Event::Keyboard(KeyReleased {
                key: Key::Named(Named::F4),
                modifiers: Modifiers::ALT,
                ..
            }) => Some(exit()),
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
