use std::time::Instant;

use bytes::Bytes;
use iced::{
    Element,
    Length::Fill,
    Renderer, Theme,
    alignment::Vertical,
    widget::{
        Column, Container, MouseArea, Row, Space, button, center, column, container, mouse_area,
        row,
        scrollable::{Direction, Scrollbar},
        space, stack, text, text_input,
        tooltip::Position,
    },
};
use log::info;
use rand::RngCore;

use crate::{
    ImgHandle, TaskHandle,
    app::{
        iced_app::{CoverUI, Message},
        song_img::{ImgHash, ImgId, SongImg},
        styles::{
            button_st, filler_st, image_hover_st, image_selected_st, img_scroll_st, input_st,
            item_cont_st, preview_box_st, select_menu_st,
        },
        view::{BTN_SIZE, INNER_TEXT_SIZE, TEXT_SIZE},
    },
    parser::file_parser::{TagData, is_rtl},
};
use iced::widget::image;
use iced::widget::scrollable;
use iced::widget::tooltip;

const CONFIRM_H: f32 = 140.0;
const MAIN_H: f32 = 320.0;
const INFO_COLUMN_GAP: f32 = 6.0;
const INFO_ROW_GAP: f32 = 6.0;
const ART_ROW_H: f32 = 200.0;
const ART_WH: f32 = ART_ROW_H - 20.0;
const INFO_LINE_H: f32 = 1.6;
const CENTER_OFFSET: f32 = 1500.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SongState {
    Confirm,
    Main,
    MainLoading,
    MainDownloading,
    Hidden,
}
impl SongState {
    fn state_to_h(&self) -> f32 {
        match self {
            SongState::Confirm => CONFIRM_H,
            SongState::Main => MAIN_H,
            SongState::MainLoading => MAIN_H,
            SongState::MainDownloading => MAIN_H,
            _ => 0.0,
        }
    }
}

/// Hash to check, when async queue return data
pub type SongHash = u64;
pub type SongId = usize;
pub type GroupSize = i32;

#[derive(Debug, Clone)]
pub struct Song {
    pub tag_data: TagData,
    pub state: SongState,
    pub queue_handle: Option<TaskHandle>,
    pub original_art: Option<ImgHandle>,
    pub hash: SongHash,
    pub menu_img: ImgHash,
    pub selected_img: ImgHash,
    /// y out of x
    pub sources_finished: (i32, i32),
    // TODO: turn this into vec of vecs
    /// size of groups, used to sort imgs when new one is added
    /// ```text
    /// [ |----gr1 = 3 ---|,|gr2 = 2 -| ]
    /// [ img1, img2, img3, img4, img5, img6 ]
    /// ```
    pub img_groups: Vec<i32>,
    // TODO: make unsorted
    /// Images ordered for display
    pub imgs: Vec<SongImg>,
}

impl Song {
    pub fn new(tag_data: TagData) -> Self {
        Self {
            state: SongState::Confirm,
            queue_handle: None,
            original_art: SongImg::original_image_preview(&tag_data),
            tag_data,
            hash: rand::rng().next_u64(),
            menu_img: 0,
            sources_finished: (0, 0),
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

        // Fix to a performance issue that never existed
        //
        // Calculate list height beforehand
        // let mut real_h = 0.0;
        // for i in 0..ui.state.songs.len() {
        //     real_h += ui.state.songs[i].state.state_to_h();
        // }
        // let pos = ui.state.list_scroll;
        // let center = real_h * pos;
        // let start = f32::max(center - CENTER_OFFSET, 0.0);
        // let end = f32::min(center + CENTER_OFFSET, real_h);

        // Draw empty boxes when item is not on screen
        // let mut real_h = 0.0;
        let mut sub_list: Vec<iced::Element<'_, _, _, _>> =
            Vec::with_capacity(ui.state.songs.len());
        for i in 0..ui.state.songs.len() {
            let h = ui.state.songs[i].state.state_to_h();
            if h > 0.0 {
                // if real_h < start || real_h > end {
                sub_list.push(Self::generate_list_item(i, ui).into());
                //         } else {
                //             sub_list.push(Self::generate_list_item(i, ui).into());
                //         }
            }
            //     real_h += h;
        }
        // info!("{} {} {} {} {real_h} ", pos, center, start, end);

        list.extend(sub_list)
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
        let theme = ui.theme.as_ref().unwrap();
        let palette = theme.extended_palette();
        let mut path_str = this.tag_data.path.as_path().to_string_lossy().to_string();
        limit_path(&mut path_str);

        let artist = this.tag_data.artist.clone().unwrap_or("".to_string());
        let artist = if is_rtl(&artist) {
            Element::from(
                text(artist)
                    .color(palette.background.base.text)
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
                    .color(palette.background.base.text)
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
                    .color(palette.background.base.text)
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

        let sources_label = text("finished:")
            .size(TEXT_SIZE)
            .height(BTN_SIZE)
            .color(palette.background.strong.text)
            .line_height(INFO_LINE_H);

        let sources = text(format!(
            "[{}/{}]",
            this.sources_finished.0, this.sources_finished.1
        ))
        .height(BTN_SIZE)
        .width(Fill)
        .line_height(INFO_LINE_H)
        .size(TEXT_SIZE);

        let path_label = text("path:")
            .size(TEXT_SIZE)
            .height(BTN_SIZE)
            .color(palette.background.strong.text)
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
                            .color(palette.background.strong.text)
                            .line_height(INFO_LINE_H),
                        text("album:")
                            .size(TEXT_SIZE)
                            .height(BTN_SIZE)
                            .color(palette.background.strong.text)
                            .line_height(INFO_LINE_H),
                        text("artist:")
                            .size(TEXT_SIZE)
                            .color(palette.background.strong.text)
                            .height(BTN_SIZE)
                            .line_height(INFO_LINE_H),
                    ]
                    .spacing(INFO_COLUMN_GAP),
                    column![
                        path,
                        row![
                            column![
                                container(title).height(BTN_SIZE),
                                container(album).height(BTN_SIZE),
                                container(artist).height(BTN_SIZE),
                            ]
                            .spacing(INFO_COLUMN_GAP),
                            if let Some(cover) = &ui.state.songs[id].original_art {
                                row![
                                    space().width(INFO_COLUMN_GAP).height(1),
                                    container(image(cover))
                                ]
                            } else {
                                row![]
                            }
                        ]
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
                    .width(80)
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
                        .push(row![sources_label, sources].spacing(INFO_ROW_GAP))
                        .push(
                            text("Update tags (TODO)")
                                .size(TEXT_SIZE)
                                .color(palette.background.strong.text)
                                .height(BTN_SIZE)
                                .line_height(INFO_LINE_H)
                        )
                        .spacing(INFO_COLUMN_GAP)
                        .height(Fill),
                    column![
                        btn("accept selected")
                            .width(Fill)
                            .style(button_st)
                            .on_press(ApplySelectedPressed(id)),
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
                .spacing(INFO_COLUMN_GAP),
            ])
            .height(MAIN_H),
            SongState::MainDownloading => container(
                text("Downloading...")
                    .center()
                    .size(50)
                    .height(Fill)
                    .color(palette.background.weak.text)
                    .width(400),
            )
            .height(MAIN_H),

            SongState::MainLoading => container(
                text("Loading...")
                    .center()
                    .size(50)
                    .color(palette.background.weak.text)
                    .height(Fill)
                    .width(400),
            )
            .height(MAIN_H),
            SongState::Hidden => panic!("Cannot draw hidden song"),
        };
        cont.style(item_cont_st).width(Fill).padding(10)
    }

    fn image_row<'a>(ui: &CoverUI, id: SongId) -> iced::widget::Scrollable<'a, Message> {
        let mut row = Row::new().height(ART_ROW_H);
        let this = &ui.state.songs[id];

        for i in 0..this.imgs.len() {
            row = row.push(Self::image_box(ui, id, i));
        }

        if this.imgs.is_empty() {
            row = row.push(
                container(
                    text("Searching...")
                        .center()
                        .size(50)
                        .height(Fill)
                        .width(400),
                )
                .style(filler_st),
            );
        }
        scrollable(row)
            .direction(Direction::Horizontal(
                Scrollbar::new().margin(0).scroller_width(15),
            ))
            .width(Fill)
            .spacing(10)
            .style(img_scroll_st)
    }

    fn image_box<'a>(ui: &CoverUI, id: SongId, img_iter: ImgId) -> MouseArea<'a, Message> {
        let this = &ui.state.songs[id];
        let img = &this.imgs[img_iter];
        let border = this.selected_img == img.hash;
        let mut info_col = Column::new().spacing(INFO_ROW_GAP - 5.0);

        let strategy = if ui.state.img_settings.square {
            iced::ContentFit::Cover
        } else {
            iced::ContentFit::Contain
        };
        if let Some((w, h)) = img.orig_res {
            info_col = info_col.push(
                text(format!("original resolution: {}x{}", w, h))
                    .size(INNER_TEXT_SIZE)
                    .center()
                    .width(Fill),
            )
        }
        info_col = info_col.push(
            text(format!("source: {}", img.src))
                .size(INNER_TEXT_SIZE)
                .center()
                .width(Fill),
        );
        info_col = info_col.push(
            text(img.feedback.to_string())
                .size(INNER_TEXT_SIZE)
                .wrapping(text::Wrapping::Word)
                .center()
                .width(Fill),
        );
        let mut cont = container(stack![
            center(
                image(img.preview.as_ref().unwrap())
                    .content_fit(strategy)
                    .width(ART_WH)
                    .height(ART_WH),
            )
            .padding(10),
            if this.menu_img == img.hash {
                center(
                    container(
                        column![
                            center(
                                column![
                                    button(text("select").size(INNER_TEXT_SIZE).center())
                                        .on_press(Message::ImgSelect(id, img.hash))
                                        .height(BTN_SIZE)
                                        .width(70)
                                        .style(button_st),
                                    button(text("preview").size(INNER_TEXT_SIZE).center())
                                        .on_press(Message::ImgPreviewOpen(id, img_iter))
                                        .height(BTN_SIZE)
                                        .width(70)
                                        .style(button_st),
                                ]
                                .spacing(INFO_ROW_GAP)
                            )
                            .width(Fill),
                            container(
                                tooltip(
                                    text("about...").center().size(INNER_TEXT_SIZE),
                                    container(info_col)
                                        .max_width(500)
                                        .padding(4)
                                        .style(select_menu_st),
                                    Position::FollowCursor
                                )
                                .gap(10)
                                .snap_within_viewport(true)
                            )
                            .center_x(Fill)
                        ]
                        .width(ART_WH)
                        .height(ART_WH)
                        .padding(20)
                        .spacing(20),
                    )
                    .style(image_hover_st),
                )
                .padding(10)
            } else {
                center(space())
            },
        ]);
        if border {
            cont = cont.style(image_selected_st);
        }
        mouse_area(cont.width(ART_ROW_H).height(ART_ROW_H))
            .on_exit(Message::ImgMenuToggle(false, id, img.hash))
            .on_enter(Message::ImgMenuToggle(true, id, img.hash))
            .on_press(Message::ImgMenuToggle(true, id, img.hash))
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
