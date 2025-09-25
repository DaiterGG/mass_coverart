use iced::{
    Element,
    Length::{Fill, FillPortion},
    Renderer, Theme,
    alignment::Vertical,
    widget::{
        Column, Container, Row, button, column, container,
        image::Handle,
        row,
        scrollable::{Direction, Scrollbar},
        text, text_input,
    },
};

use crate::{
    api::queue::ImgData,
    app::{
        iced_app::{BTN_SIZE, CoverUI, INNER_TEXT_SIZE, Message, TEXT_SIZE},
        styles::{button_st, input_st, item_cont_st, list_scroll_st},
    },
    parser::file_parser::{FileParser, TagData},
};
use iced::widget::image;
use iced::widget::scrollable;

const CONFIRM_H: f32 = 140.0;
const MAIN_H: f32 = 300.0;
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

pub type SongId = usize;
pub struct Song {
    pub tag_data: TagData,
    pub state: SongState,
    pub imgs: Vec<ImgData>,
}

impl Song {
    pub fn new(tag_data: TagData) -> Self {
        Self {
            tag_data,
            state: SongState::Confirm,
            imgs: Vec::new(),
        }
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
                list = list.push(ui.state.songs[i].generate_list_item(i, ui));
            } else if h > 0.0 {
                list = list.push(container("").height(h));
            }
            real_h += h;
        }
        list
    }
    pub fn generate_list_item<'a>(
        &self,
        iter_id: usize,
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

        let mut path_str = self.tag_data.path.as_path().to_string_lossy().to_string();
        limit_path(&mut path_str);

        let artist = self.tag_data.artist.clone().unwrap_or("".to_string());
        let artist = if FileParser::is_rtl(&artist) {
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
                    .on_input(move |s| ArtistInput(iter_id, s))
                    .size(INNER_TEXT_SIZE),
            )
        };

        let title = self.tag_data.title.clone().unwrap_or("".to_string());
        let title = if FileParser::is_rtl(&title) {
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
                    .on_input(move |s| TitleInput(iter_id, s))
                    .size(INNER_TEXT_SIZE),
            )
        };

        let album = self.tag_data.album.clone().unwrap_or("".to_string());
        let album = if FileParser::is_rtl(&album) {
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
                    .on_input(move |s| AlbumInput(iter_id, s))
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
        let cont = match self.state {
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
                            .on_press(ConfirmSong(iter_id)),
                        btn("close")
                            .width(Fill)
                            .style(button_st)
                            .on_press(DiscardSong(iter_id)),
                    ]
                    .spacing(INFO_COLUMN_GAP)
                    .width(80),
                ]
                .align_y(Vertical::Center)
                .spacing(INFO_ROW_GAP),
            )
            .height(CONFIRM_H),
            SongState::Main => container(row![
                Column::new()
                    .push(row![path_label, path].spacing(INFO_ROW_GAP))
                    .push(self.image_row())
                    // row![],
                    //
                    .spacing(INFO_COLUMN_GAP)
                    .height(Fill),
                column![
                    btn("accept selected")
                        .width(Fill)
                        .style(button_st)
                        .on_press(Exit),
                    btn("back to tags")
                        .width(Fill)
                        .style(button_st)
                        .on_press(Exit),
                    btn("close").width(Fill).style(button_st).on_press(Exit),
                ]
                .spacing(INFO_COLUMN_GAP)
                .width(80),
            ])
            .height(MAIN_H),
            SongState::Hidden => panic!("Cannot draw hidden song"),
        };
        cont.style(item_cont_st).width(Fill).padding(10)
    }

    fn image_row<'a>(&self) -> iced::widget::Scrollable<'a, Message> {
        let mut row = Row::new();
        for img in &self.imgs {
            match img {
                ImgData::Path(p) => {}
                ImgData::PreviewPathUrl(path, url) => {}
                ImgData::Bytes(bytes, format) => {
                    row = row.push(image(bytes).content_fit(iced::ContentFit::Contain));
                }
            }
        }
        scrollable(row)
            .direction(Direction::Horizontal(
                Scrollbar::new().margin(0).scroller_width(15),
            ))
            .width(Fill)
            .height(ART_ROW_H)
            .spacing(10)
            // .on_scroll(|v| Offset(v.relative_offset().y))
            .style(list_scroll_st)
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
