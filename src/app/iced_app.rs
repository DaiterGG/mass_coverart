use std::{path::PathBuf, sync::Arc, time::Duration, vec};

use bytes::Bytes;
use iced::{
    Element, Event, Subscription, Task, Theme, event, exit,
    keyboard::{Event::KeyReleased, Key, key::Named},
    window::{self, icon},
};
use log::{error, info, warn};
use reqwest::Client;
use rfd::{AsyncFileDialog, FileHandle};
use tokio::{sync::Semaphore, time::sleep};

use crate::{
    ImgHandle,
    api::{
        queue::{
            Queue, QueueMessage,
            Source::{self, YoutubeAlbum},
            TagsInput,
        },
        shared,
    },
    app::{
        img::{ImageProgress, ImageSettings, ImgFormat, ImgId, SongImg},
        song::{OrigArt, Song, SongHash, SongId, SongState},
        styles::*,
        view::{PreviewState, REGEX_LIM, view},
    },
    parser::file_parser::{self, ParseSettings, RegexType, get_tags_data},
};
#[derive(Clone)]
pub enum Message {
    FileOpen,
    FolderOpen,
    PathOpenEnd(Option<Vec<FileHandle>>),
    GotPath(Vec<FileHandle>),
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
    GotArt(SongId, SongHash, SongImg),
    AutoModTrigger,
    ProcessedArt(SongId, SongHash, SongImg),
    Scroll(f32),
    ImgSelect(SongId, ImgId),
    ImgPreviewOpen(SongId, ImgId),
    ImgPreview(SongId, ImgId),
    ImgPreviewSet(PreviewState),
    DecodePreview(Bytes, ImgFormat, SongId, ImgId),
    ImgMenuToggle(bool, SongId, ImgId),
    TagToggle(SongId, usize),
    LoadOrigImg(SongId),
    SetOrigImg(ImgHandle, SongId, SongHash),
    SaveImgLocally(SongId, ImgId),
    SaveImgLocallyEnd(Option<FileHandle>, SongId, ImgId),
    RemoveImageFromFile(SongId),
    OrigImageHover(bool, SongId),
    AddLocalImage(SongId),
    AddLocalImageMiddle(SongId, SongHash, Option<Vec<FileHandle>>),
    AddLocalImageEnd(SongId, SongHash, Vec<u8>, String),
    SelectFirst(SongId),
    CopyImgAbout(SongId, ImgId),
    ExitAbout,
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
    pub preview_client: Client,
    pub ui_blocked: bool,
    pub ui_loading: bool,
    pub parse_settings: ParseSettings,
    pub _init_size: (f32, f32),
    pub auto_mod: bool,
    pub auto_mod_current_song: usize,
    pub img_settings: ImageSettings,
    pub copied_message: bool,
}
pub fn song_is_invalid(st: &State, id: SongId, hash: SongHash) -> bool {
    if id >= st.songs.len() || st.songs[id].hash != hash {
        error!("attempt to access invalid song {}", id);
        true
    } else {
        false
    }
}
pub struct CoverUI {
    pub state: State,
    pub decode_sem: Arc<Semaphore>,
    pub theme: Option<Theme>,
}
impl CoverUI {
    pub fn init(init_size: (f32, f32)) -> (Self, Task<Message>) {
        let t = window::oldest()
            .and_then(move |id| {
                let icon = icon::from_file_data(
                    include_bytes!("../../resources/icon.png"),
                    Some(::image::ImageFormat::Png),
                )
                .unwrap();
                window::set_icon(id, icon)
            })
            .chain(Task::done(Message::Start));
        (
            Self {
                theme: Some(miasma_theme()),
                decode_sem: Arc::new(Semaphore::new(1)),
                state: State {
                    _init_size: init_size,
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
                return Task::done(GotPath(vec![PathBuf::new().join("./foo/sub/").into()]))
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
                         ImgFormat::Jpeg,
                         ImageProgress::Raw(Bytes::from_static(include_bytes!("../../foo/2.jpg"))),
                        YoutubeAlbum,
                        "AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA AA ".to_string()
                        ))))
                )
                .chain(Task::future(async {
                    ConfirmSongIfNot(0)
                }));
            }
            Exit => {
                return exit();
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
            FileOpen => {
                self.state.ui_blocked = true;

                let files = AsyncFileDialog::new()
                    .set_title("Open image to add")
                    .add_filter("Audio", &["mp3", "m4a"])
                    .add_filter("All", &["*"])
                    .set_directory("/")
                    .pick_files();
                return Task::perform(files, PathOpenEnd);
            }
            FolderOpen => {
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

            ImgSelect(song_id, img_id) => {
                self.state.songs[song_id].selected_img = Some(img_id);
                return Task::done(ImgMenuToggle(false, song_id, img_id));
            }
            ImgMenuToggle(enter, song_id, img_id) => {
                if enter {
                    let now = self.state.songs[song_id].menu_img;
                    if now != Some(img_id) {
                        self.state.songs[song_id].menu_img = Some(img_id);
                    }
                } else {
                    let now = self.state.songs[song_id].menu_img;
                    if now == Some(img_id) {
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
                let client = self.state.preview_client.clone();
                if let ImageProgress::Preview(urls) = &self.state.songs[song_i].imgs[img_id].image {
                    let format = ImgFormat::from_url(urls.first().unwrap());
                    let urls = urls.to_vec();
                    let (t, h) = Task::perform(
                        async move { shared::get_img(&client, urls).await },
                        move |res| {
                            if let Ok(bytes) = res {
                                DecodePreview(bytes, format, song_i, img_id)
                            } else {
                                ImgPreviewSet(PreviewState::Error)
                            }
                        },
                    )
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
            ImgPreview(song_id, img_id) => {
                let state = self.state.songs[song_id].imgs[img_id]
                    .final_img_preview(self.state.img_settings);
                self.state.preview_img = PreviewState::Display(state, song_id, img_id);
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

                if let Err(e) = song.selected_tags.apply_selected(&mut song.tag_data) {
                    error!("{}", e);
                    return Task::done(DiscardSong(song_id));
                }

                if let Some(img_id) = song.selected_img {
                    let img = &mut song.imgs[img_id];
                    let client = self.state.preview_client.clone();
                    let task = if let ImageProgress::Preview(urls) = &img.image {
                        song.state = SongState::MainDownloading;
                        let format = ImgFormat::from_url(urls.first().unwrap());
                        let urls = urls.to_vec();
                        Task::perform(
                            async move { shared::get_img(&client, urls).await },
                            move |res| {
                                if let Ok(bytes) = res {
                                    DecodeAccept(bytes, format, song_id)
                                } else {
                                    error!("{}", res.unwrap_err());
                                    AcceptFailed(song_id)
                                }
                            },
                        )
                    } else {
                        song.state = SongState::MainLoading;
                        Task::done(ApplySelected(song_id))
                    };
                    return task.chain(Task::done(AutoModTrigger));
                } else {
                    return Task::done(GoBack(song_id));
                }
            }

            AcceptFailed(song_id) => {
                self.state.songs[song_id].state = SongState::Main;
            }
            DecodeAccept(bytes, format, song_id) => {
                let song = &mut self.state.songs[song_id];

                if let Some(img_id) = song.selected_img {
                    self.state.songs[song_id].state = SongState::MainLoading;
                    let res =
                        self.state.songs[song_id].imgs[img_id].preview_to_decoded(bytes, format);

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
                if let Err(e) = file_parser::apply_selected(self, song_id) {
                    error!("{}", e);
                    return Task::done(DiscardSong(song_id));
                }
                return Task::done(GoBack(song_id));
            }
            ConfirmSongIfNot(id) => {
                if self.state.songs.len() > id && self.state.songs[id].state == SongState::Confirm {
                    let song = &mut self.state.songs[id];
                    let info = TagsInput::from_data(id, song.hash, &song.tag_data);
                    song.state = SongState::Main;
                    let (q, handle) = Queue::init(info);
                    song.queue_handle = Some(handle);

                    song.new_tags
                        .extend(file_parser::find_edited_tags(&song.tag_data));
                    song.new_tags.extend(song.tags_from_regex.clone());
                    return q;
                }
            }
            GoBackDiscard(id) => return Task::done(GoBack(id)).chain(Task::done(DiscardSong(id))),

            GoBack(id) => {
                self.state.songs[id].state = SongState::Confirm;

                self.state.songs[id].reset();
                return Task::done(AutoModTrigger);
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
                        return Task::done(Message::GotArt(id, hash, output));
                    }
                    SetSources(num, out_of) => {
                        self.state.songs[id].sources_finished = (num, out_of);
                        if self.state.auto_mod && num == out_of {
                            let mut task = Task::none();
                            if self.state.songs[id].state == SongState::Main
                                && !self.state.songs[id].imgs.is_empty()
                                && self.state.songs[id].selected_img.is_none()
                            {
                                task = task.chain(Task::done(SelectFirst(id)));
                            }
                            return task.chain(Task::done(AutoModTrigger));
                        }
                    }
                    SourceFinished => {
                        let now = self.state.songs[id].sources_finished.0;
                        self.state.songs[id].sources_finished = (now + 1, Queue::TOTAL_SOURCES)
                    }
                }
            }
            GotArt(id, hash, img) => {
                return Task::perform(
                    SongImg::decode_and_sample(img, self.decode_sem.clone()),
                    move |res| {
                        if let Ok(ok) = res {
                            ProcessedArt(id, hash, ok)
                        } else {
                            error!("img was not decoded and sapmled: {},", res.unwrap_err());
                            Nothing
                        }
                    },
                );
            }
            AutoModTrigger => {
                if self.state.auto_mod {
                    let cur = self.state.auto_mod_current_song;
                    // current song in process right now
                    if cur < self.state.songs.len() {
                        let song = &mut self.state.songs[cur];
                        let (a, b) = song.sources_finished;
                        if song.state == SongState::Main && a != b {
                            return Task::none();
                        }
                    }
                    for i in (cur + 1)..self.state.songs.len() {
                        if self.state.songs[i].state != SongState::Confirm {
                            continue;
                        }
                        self.state.auto_mod_current_song = i;
                        return Task::done(ConfirmSongIfNot(i));
                    }
                }
            }
            ProcessedArt(id, hash, output) => {
                let mut task = Task::none();
                if song_is_invalid(&self.state, id, hash) {
                    return task;
                }
                let song = &mut self.state.songs[id];
                if output.src == Source::LocalFile {
                    task = Task::done(SelectFirst(id));
                }
                let res = output.push_and_group(&mut song.img_groups, &mut song.imgs);

                let _ = res.inspect_err(|e| warn!("img was not added: {e}"));
                return task;
            }
            Scroll(scroll_uv) => {
                self.state.list_scroll = scroll_uv;
            }
            AutoModToggle(on) => {
                self.state.auto_mod = on;
                self.state.auto_mod_current_song = 0;
                return Task::done(AutoModTrigger);
            }

            LoadOrigImg(song_id) => {
                let song = &mut self.state.songs[song_id];
                song.original_art = Some(OrigArt::Loading);

                let img = song.tag_data.file.album_cover().unwrap();
                let hash = song.hash;
                return Task::perform(
                    SongImg::original_image_preview(img.data.to_owned(), img.mime_type),
                    move |res| {
                        if let Some(h) = res {
                            return SetOrigImg(h, song_id, hash);
                        }
                        Nothing
                    },
                );
            }
            SetOrigImg(handle, id, hash) => {
                if !song_is_invalid(&self.state, id, hash) {
                    self.state.songs[id].original_art = Some(OrigArt::Loaded(handle));
                }
            }
            TagToggle(id, tag_iter) => {
                let song = &mut self.state.songs[id];
                let tag = &mut song.new_tags.sorted[tag_iter];
                song.selected_tags.toggle(tag.key, Some(tag.value.clone()));
            }
            SaveImgLocally(song_id, img_id) => {
                let song = self
                    .state
                    .songs
                    .get_mut(song_id)
                    .expect("song cannot be deselected when preview open");
                let path = song.tag_data.path.as_path();
                let root = path.parent().expect("file and has root");
                let title = path.file_name().map_or("image".to_string(), |t| {
                    t.to_string_lossy()
                        .to_string()
                        .rsplit_once('.')
                        .expect("music file has extension")
                        .0
                        .to_string()
                });
                let ext = song.imgs[img_id].orig_format.to_str();
                let mut show_ext = ".".to_string();
                show_ext.push_str(ext);
                let show_ext = show_ext.to_uppercase();
                let files = AsyncFileDialog::new()
                    .set_title("Save image")
                    .set_directory(root)
                    .set_file_name(title)
                    .add_filter(show_ext, &[ext])
                    .add_filter("All", &["*"])
                    .save_file();
                return Task::perform(files, move |d| SaveImgLocallyEnd(d, song_id, img_id));
            }
            SaveImgLocallyEnd(data, song_id, img_id) => {
                if let Some(file_handle) = data {
                    self.state.songs[song_id].imgs[img_id]
                        .decoded()
                        .save(file_handle.path())
                        .unwrap();
                }
            }
            RemoveImageFromFile(song_id) => {
                let song = &mut self.state.songs[song_id];
                song.tag_data.file.remove_album_cover();
                let res = song
                    .tag_data
                    .file
                    .write_to_path(song.tag_data.path.to_str().unwrap());
                if let Err(e) = res {
                    error!("{}", e);
                    return Task::none();
                }
                song.original_art = None;
            }
            OrigImageHover(hovered, song_id) => {
                self.state.songs[song_id].original_art_hovered = hovered;
            }
            AddLocalImage(song_id) => {
                self.state.ui_blocked = true;
                let song = self
                    .state
                    .songs
                    .get_mut(song_id)
                    .expect("song cannot be deselected when preview open");
                let path = song.tag_data.path.as_path();
                let root = path.parent().expect("file and has root");
                let files = AsyncFileDialog::new()
                    .set_title("Open image to add")
                    .add_filter("Image .JPG/.JPEG, .PNG", &["jpg", "png"])
                    .add_filter("All", &["*"])
                    .set_directory(root)
                    .pick_files();
                let hash = song.hash;
                return Task::perform(files, move |d| AddLocalImageMiddle(song_id, hash, d));
            }
            AddLocalImageMiddle(id, hash, files) => {
                self.state.ui_blocked = false;
                if files.is_none() || song_is_invalid(&self.state, id, hash) {
                    return Task::none();
                }
                let mut files = files.unwrap();

                let mut task = Task::none();
                for _ in 0..files.len() {
                    let file = files.pop().unwrap();
                    let path = file.path().to_string_lossy().to_string();
                    let fut = async move { file.read().await };
                    task = task.chain(Task::perform(fut, move |vec| {
                        AddLocalImageEnd(id, hash, vec, path)
                    }));
                }
                return task;
            }
            AddLocalImageEnd(id, hash, file, path) => {
                if song_is_invalid(&self.state, id, hash) {
                    return Task::none();
                }
                let bytes = Bytes::from_owner(file);
                let img = SongImg::new(
                    ImgFormat::Jpeg,
                    ImageProgress::Raw(bytes),
                    Source::LocalFile,
                    path,
                );

                return Task::done(GotArt(id, hash, img));
            }
            SelectFirst(song_id) => {
                let song = &mut self.state.songs[song_id];
                let first_id = song.img_groups.first_in_first_group();
                return Task::done(ImgSelect(song_id, first_id));
            }
            CopyImgAbout(song_id, img_id) => {
                let about = self.state.songs[song_id].imgs[img_id].feedback.clone();
                self.state.copied_message = true;
                return iced::clipboard::write::<Message>(about);
            }
            ExitAbout => {
                self.state.copied_message = false;
            }
            _ => {
                error!("unhandled message");
            }
        }
        Task::none()
    }
    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _windows| match event {
            Event::Window(window::Event::FileDropped(path)) => {
                Some(Message::PathDropped(vec![path.into()]))
            }
            #[cfg(debug_assertions)]
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
