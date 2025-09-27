use iced::{
    Element,
    Length::Fill,
    Renderer, Theme,
    alignment::Vertical,
    widget::{
        Column, Container, MouseArea, Row, Space, button, center, column, container, mouse_area,
        row,
        scrollable::{Direction, Scrollbar},
        text, text_input,
    },
};
use rand::RngCore;

use crate::{
    TaskHandle,
    app::{
        iced_app::{CoverUI, Message},
        song_img::{ImgHash, SongImg},
        styles::{button_st, image_selected_st, img_scroll_st, input_st, item_cont_st},
        view::{BTN_SIZE, INNER_TEXT_SIZE, TEXT_SIZE},
    },
    parser::file_parser::{TagData, is_rtl},
};
use iced::widget::image;
use iced::widget::scrollable;

const CONFIRM_H: f32 = 140.0;
const MAIN_H: f32 = 350.0;
const INFO_COLUMN_GAP: f32 = 6.0;
const INFO_ROW_GAP: f32 = 6.0;
const ART_ROW_H: f32 = 200.0;
const INFO_LINE_H: f32 = 1.6;
const CENTER_OFFSET: f32 = 1500.0;

#[derive(PartialEq, Eq, Debug)]
pub enum SongState {
    Confirm,
    Main,
    Hidden,
}
impl SongState {
    fn state_to_h(&self) -> f32 {
        match self {
            SongState::Confirm => CONFIRM_H,
            SongState::Main => MAIN_H,
            _ => 0.0,
        }
    }
}

/// Hash to check, when async queue return data
pub type SongHash = u64;
pub type SongId = usize;
pub type GroupSize = i32;

pub struct Song {
    pub tag_data: TagData,
    pub state: SongState,
    pub queue_handle: Option<TaskHandle>,
    pub hash: SongHash,
    pub menu_img: ImgHash,
    pub selected_img: ImgHash,
    /// size of groups, used to sort imgs when new one is added
    /// ```text
    /// [ |----gr1 = 3 ---|,|gr2 = 2 -| ]
    /// [ img1, img2, img3, img4, img5, img6 ]
    /// ```
    pub img_groups: Vec<i32>,
    /// Images ordered for display
    pub imgs: Vec<SongImg>,
}

impl Song {
    pub fn new(tag_data: TagData) -> Self {
        Self {
            tag_data,
            state: SongState::Confirm,
            queue_handle: None,
            hash: rand::rng().next_u64(),
            menu_img: 0,
            selected_img: 0,
            img_groups: Vec::new(),
            imgs: Vec::new(),
        }
    }
    pub fn unselect(&mut self) {
        self.selected_img = 0;
    }
    pub fn menu_close(&mut self) {
        self.menu_img = 0;
    }
    pub fn generate_view_list(ui: &CoverUI) -> iced::widget::Column<'_, Message> {
        let mut list = column![].padding(8).spacing(5);

        // Calculate list height beforehand
        let mut real_h = 0.0;
        for i in 0..ui.state.songs.len() {
            real_h += ui.state.songs[i].state.state_to_h();
        }
        let pos = ui.state.list_offset;
        let center = real_h * pos;
        let start = f32::max(center - CENTER_OFFSET, 0.0);
        let end = f32::min(center + CENTER_OFFSET, real_h);

        // println!("{} {}", start, end);

        // Draw empty boxes when item is not on screen
        let mut real_h = 0.0;
        for i in 0..ui.state.songs.len() {
            let h = ui.state.songs[i].state.state_to_h();
            if real_h >= start && real_h <= end && h > 0.0 {
                list = list.push(Self::generate_list_item(i, ui));
            } else if h > 0.0 {
                list = list.push(Space::new(0, h));
            }
            real_h += h;
        }
        list
    }
    pub fn generate_list_item<'a>(
        id: SongId,
        ui: &CoverUI,
    ) -> Container<'a, Message, Theme, Renderer> {
        use Message::*;
        let h3 = |s| {
            text(s)
                .size(INNER_TEXT_SIZE)
                .width(Fill)
                .height(Fill)
                .wrapping(text::Wrapping::None)
        };

        let this = &ui.state.songs[id];
        let mut path_str = this.tag_data.path.as_path().to_string_lossy().to_string();
        limit_path(&mut path_str);

        let artist = this.tag_data.artist.clone().unwrap_or("".to_string());
        let artist = if is_rtl(&artist) {
            Element::from(
                text(artist)
                    .color(ui.theme().extended_palette().background.base.text)
                    .width(Fill)
                    .size(INNER_TEXT_SIZE),
            )
        } else {
            Element::from(
                text_input("Not found", &artist)
                    .style(input_st)
                    .width(Fill)
                    .on_input(move |s| ArtistInput(id, s))
                    .size(INNER_TEXT_SIZE),
            )
        };

        let title = this.tag_data.title.clone().unwrap_or("".to_string());
        let title = if is_rtl(&title) {
            Element::from(
                text(title)
                    .color(ui.theme().extended_palette().background.base.text)
                    .width(Fill)
                    .size(INNER_TEXT_SIZE),
            )
        } else {
            Element::from(
                text_input("Not found", &title)
                    .style(input_st)
                    .width(Fill)
                    .on_input(move |s| TitleInput(id, s))
                    .size(INNER_TEXT_SIZE),
            )
        };

        let album = this.tag_data.album.clone().unwrap_or("".to_string());
        let album = if is_rtl(&album) {
            Element::from(
                text(album)
                    .color(ui.theme().extended_palette().primary.base.text)
                    .width(Fill)
                    .size(INNER_TEXT_SIZE),
            )
        } else {
            Element::from(
                text_input("Not found", &album)
                    .style(input_st)
                    .width(Fill)
                    .on_input(move |s| AlbumInput(id, s))
                    .size(INNER_TEXT_SIZE),
            )
        };
        let path_label = text("path:")
            .size(TEXT_SIZE)
            .height(BTN_SIZE)
            .color(ui.theme().extended_palette().background.weak.text)
            .line_height(INFO_LINE_H);
        let path = text(path_str)
            .height(BTN_SIZE)
            .width(Fill)
            .line_height(INFO_LINE_H)
            .size(TEXT_SIZE);
        let btn = |s| button(h3(s).center()).clip(true).height(BTN_SIZE);
        let cont = match this.state {
            SongState::Confirm => container(
                row![
                    column![
                        path_label,
                        text("title:")
                            .size(TEXT_SIZE)
                            .height(BTN_SIZE)
                            .color(ui.theme().extended_palette().background.weak.text)
                            .line_height(INFO_LINE_H),
                        text("album:")
                            .size(TEXT_SIZE)
                            .height(BTN_SIZE)
                            .color(ui.theme().extended_palette().background.weak.text)
                            .line_height(INFO_LINE_H),
                        text("artist:")
                            .size(TEXT_SIZE)
                            .color(ui.theme().extended_palette().background.weak.text)
                            .height(BTN_SIZE)
                            .line_height(INFO_LINE_H),
                    ]
                    .spacing(INFO_COLUMN_GAP),
                    column![
                        path,
                        container(title).height(BTN_SIZE),
                        container(album).height(BTN_SIZE),
                        container(artist).height(BTN_SIZE),
                    ]
                    .spacing(INFO_COLUMN_GAP),
                    column![
                        btn("confirm")
                            .width(Fill)
                            .style(button_st)
                            .on_press(ConfirmSong(id)),
                        btn("remove")
                            .width(Fill)
                            .style(button_st)
                            .on_press(DiscardSong(id)),
                    ]
                    .spacing(INFO_COLUMN_GAP)
                    .width(80),
                ]
                .align_y(Vertical::Center)
                .spacing(INFO_ROW_GAP),
            )
            .height(CONFIRM_H),
            SongState::Main => container(column![
                row![path_label, path].spacing(INFO_ROW_GAP),
                row![
                    Column::new()
                        .push(Self::image_row(ui, id))
                        .push(
                            text("Update tags")
                                .size(TEXT_SIZE)
                                .color(ui.theme().extended_palette().background.weak.text)
                                .height(BTN_SIZE)
                                .line_height(INFO_LINE_H)
                        )
                        // row![],
                        //
                        .spacing(INFO_COLUMN_GAP)
                        .height(Fill),
                    column![
                        btn("accept selected")
                            .width(Fill)
                            .style(button_st)
                            .on_press(Accept(id)),
                        btn("back to tags")
                            .width(Fill)
                            .clip(true)
                            .style(button_st)
                            .on_press(GoBack(id)),
                        btn("remove")
                            .width(Fill)
                            .style(button_st)
                            .on_press(GoBackDiscard(id)),
                    ]
                    .spacing(INFO_COLUMN_GAP)
                    .width(120),
                ]
                .spacing(INFO_COLUMN_GAP)
            ])
            .height(MAIN_H),
            SongState::Hidden => panic!("Cannot draw hidden song"),
        };
        cont.style(item_cont_st).width(Fill).padding(10)
    }

    fn image_row<'a>(ui: &CoverUI, id: SongId) -> iced::widget::Scrollable<'a, Message> {
        let mut row = Row::new();
        let this = &ui.state.songs[id];

        for i in 0..this.imgs.len() {
            row = row.push(Self::image_box(ui, id, i));
        }
        scrollable(row)
            .direction(Direction::Horizontal(
                Scrollbar::new().margin(0).scroller_width(15),
            ))
            .width(Fill)
            .spacing(10)
            .style(img_scroll_st)
    }

    fn image_box<'a>(ui: &CoverUI, id: SongId, img_iter: usize) -> MouseArea<'a, Message> {
        let this = &ui.state.songs[id];
        let img = &this.imgs[img_iter];
        let border = this.selected_img == img.hash;
        let (w, h) = img.resolution();

        let mut cont = if this.menu_img == img.hash {
            center(
                column![
                    center(
                        column![
                            button(text("select").size(INNER_TEXT_SIZE).center())
                                .on_press(Message::ImgSelect(id, img.hash))
                                .height(BTN_SIZE)
                                .width(70)
                                .style(button_st),
                            button(text("preview").size(INNER_TEXT_SIZE).center())
                                .on_press(Message::ImgPreview(id, img_iter))
                                .height(BTN_SIZE)
                                .width(70)
                                .style(button_st),
                        ]
                        .spacing(INFO_ROW_GAP)
                    )
                    .width(Fill),
                    column![
                        text(format!("{}x{}", w, h))
                            .size(INNER_TEXT_SIZE)
                            .center()
                            .width(Fill),
                        text(format!("{}", img.src))
                            .size(INNER_TEXT_SIZE)
                            .center()
                            .width(Fill),
                    ]
                    .spacing(INFO_ROW_GAP)
                ]
                .padding(20)
                .spacing(20),
            )
        } else {
            center(image(img.handle.clone()).content_fit(iced::ContentFit::Contain)).padding(10)
        };
        if border {
            cont = cont.style(image_selected_st);
        }
        mouse_area(cont.width(ART_ROW_H).height(ART_ROW_H))
            .on_press(Message::ImgMenuToggle(true, id, img.hash))
            .on_exit(Message::ImgMenuToggle(false, id, img.hash))
    }
}

fn limit_path(path_str: &mut String) {
    if path_str.len() > 70 {
        let mut i = path_str.len() - 70;
        let mut opt = None;
        while opt.is_none() {
            opt = path_str.split_at_checked(i);
            i += 1;
        }
        *path_str = opt.unwrap().1.to_string();
        path_str.insert_str(0, "... ");
    }
}
