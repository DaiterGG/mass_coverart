use crate::{
    ImgHandle,
    app::{
        iced_app::{CoverUI, Message, song_is_invalid},
        img::ImgId,
        song::{OrigArt, SongId, SongState},
        styles::{
            button_st, filler_st, image_hover_st, image_selected_st, img_scroll_st, input_st,
            item_cont_st, select_menu_st, tag_st,
        },
        view::{BTN_HEIGHT, INNER_TEXT_SIZE, TEXT_SIZE},
    },
    parser::file_parser::is_rtl,
};
use iced::widget::image;
use iced::widget::scrollable;
use iced::widget::tooltip;
use iced::{
    Element,
    Length::Fill,
    Renderer, Theme,
    alignment::Vertical,
    widget::{
        Button, Column, MouseArea, Row, Sensor, button, center, column, container, mouse_area, row,
        scrollable::{Direction, Scrollbar},
        space, stack, text, text_input,
        tooltip::Position,
    },
};

pub const CONFIRM_H: f32 = 140.0;
pub const MAIN_H: f32 = 360.0;
const INFO_COLUMN_GAP: f32 = 6.0;
const INFO_ROW_GAP: f32 = 6.0;
const ART_ROW_H: f32 = 200.0;
const ART_WH: f32 = ART_ROW_H - 40.0;
const TAG_H: f32 = 30.0;
const TAG_SPACING: f32 = 10.0;
const INFO_LINE_H: f32 = 1.6;
const CENTER_OFFSET: f32 = 1500.0;

pub fn generate_view_list(ui: &CoverUI) -> iced::widget::Column<'_, Message> {
    let list = column![].padding(8).spacing(5);

    // Calculate list height beforehand
    let mut real_h = 0.0;
    for i in 0..ui.state.songs.len() {
        real_h += ui.state.songs[i].state.state_to_h();
    }
    let pos = ui.state.list_scroll;
    let center = real_h * pos;
    let start = f32::max(center - CENTER_OFFSET, 0.0);
    let end = f32::min(center + CENTER_OFFSET, real_h);

    let mut real_h = 0.0;

    let mut sub_list: Vec<iced::Element<'_, _, _, _>> = Vec::with_capacity(ui.state.songs.len());
    for i in 0..ui.state.songs.len() {
        let h = ui.state.songs[i].state.state_to_h();
        if h > 0.0 {
            if real_h < start || real_h > end {
                sub_list.push(generate_list_item(i, ui, true).into());
            } else {
                sub_list.push(generate_list_item(i, ui, false).into());
            }
            real_h += h;
        }
    }

    list.extend(sub_list)
}
pub fn generate_list_item<'a>(
    id: SongId,
    ui: &CoverUI,
    hide: bool,
) -> Row<'a, Message, Theme, Renderer> {
    use Message::*;
    if hide {
        if ui.state.songs[id].state == SongState::Confirm {
            return row![space().height(CONFIRM_H).width(1)];
        } else {
            return row![container(space().height(MAIN_H).width(1))];
        }
    }
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
        .height(BTN_HEIGHT)
        .color(palette.background.strong.text)
        .line_height(INFO_LINE_H);

    let sources = text(format!(
        "[{}/{}]",
        this.sources_finished.0, this.sources_finished.1
    ))
    .height(BTN_HEIGHT)
    .width(Fill)
    .line_height(INFO_LINE_H)
    .size(TEXT_SIZE);

    let path_label = text("path:")
        .size(TEXT_SIZE)
        .height(BTN_HEIGHT)
        .color(palette.background.strong.text)
        .line_height(INFO_LINE_H);
    let path = text(path_str)
        .height(BTN_HEIGHT)
        .width(Fill)
        .line_height(INFO_LINE_H)
        .size(TEXT_SIZE);
    let btn = |s| button(h3(s).center()).clip(true).height(BTN_HEIGHT);

    let cont = match this.state {
        SongState::Confirm => container(
            row![
                column![
                    path_label,
                    text("title:")
                        .size(TEXT_SIZE)
                        .height(BTN_HEIGHT)
                        .color(palette.background.strong.text)
                        .line_height(INFO_LINE_H),
                    text("album:")
                        .size(TEXT_SIZE)
                        .height(BTN_HEIGHT)
                        .color(palette.background.strong.text)
                        .line_height(INFO_LINE_H),
                    text("artist:")
                        .size(TEXT_SIZE)
                        .color(palette.background.strong.text)
                        .height(BTN_HEIGHT)
                        .line_height(INFO_LINE_H),
                ]
                .spacing(INFO_COLUMN_GAP),
                column![
                    path,
                    row![
                        column![
                            container(title).height(BTN_HEIGHT),
                            container(album).height(BTN_HEIGHT),
                            container(artist).height(BTN_HEIGHT),
                        ]
                        .spacing(INFO_COLUMN_GAP),
                        if let Some(cover) = &ui.state.songs[id].original_art {
                            if let OrigArt::Loaded(art) = cover {
                                row![
                                    space().width(INFO_COLUMN_GAP).height(1),
                                    orig_img(ui, id, art)
                                ]
                            } else if *cover == OrigArt::Loading {
                                row![]
                            } else {
                                row![
                                    Sensor::new(space().width(1).height(Fill))
                                        .on_show(move |_| LoadOrigImg(id))
                                ]
                            }
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
                        .on_press(ConfirmSongIfNot(id)),
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
                    .push(image_row(ui, id))
                    .push(row![sources_label, sources].spacing(INFO_ROW_GAP))
                    .push(
                        text("update tags:")
                            .size(TEXT_SIZE)
                            .color(palette.background.strong.text)
                            .height(BTN_HEIGHT)
                            .line_height(INFO_LINE_H)
                    )
                    .push(tags_list(ui, id))
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
                    btn("add local image")
                        .width(Fill)
                        .clip(true)
                        .style(button_st)
                        .on_press(AddLocalImage(id)),
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
    row![
        cont.style(item_cont_st).width(Fill).padding(10),
        space().width(20).height(20)
    ]
}

fn image_row<'a>(ui: &CoverUI, id: SongId) -> iced::widget::Scrollable<'a, Message> {
    let mut row = Row::new();
    let this = &ui.state.songs[id];

    for i in this.img_groups.flat() {
        row = row.push(image_box(ui, id, *i));
    }

    if this.imgs.is_empty() {
        if this.sources_finished.0 == this.sources_finished.1 {
            row = row.push(
                container(text("Not found").center().size(50).height(Fill).width(400))
                    .style(filler_st),
            );
        } else {
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
    }
    scrollable(row)
        .direction(Direction::Horizontal(
            Scrollbar::new().margin(0).scroller_width(15),
        ))
        .height(ART_ROW_H)
        .width(Fill)
        .spacing(10)
        .style(img_scroll_st)
}

fn image_box<'a>(ui: &CoverUI, id: SongId, img_iter: ImgId) -> MouseArea<'a, Message> {
    let this = &ui.state.songs[id];
    let img = &this.imgs[img_iter];
    let border = this.selected_img == Some(img_iter);
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
        if this.menu_img == Some(img_iter) {
            center(
                container(
                    column![
                        center(
                            column![
                                button(text("select").size(INNER_TEXT_SIZE).center())
                                    .on_press(Message::ImgSelect(id, img_iter))
                                    .height(BTN_HEIGHT)
                                    .width(70)
                                    .style(button_st),
                                button(text("preview").size(INNER_TEXT_SIZE).center())
                                    .on_press(Message::ImgPreviewOpen(id, img_iter))
                                    .height(BTN_HEIGHT)
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
        .on_exit(Message::ImgMenuToggle(false, id, img_iter))
        .on_enter(Message::ImgMenuToggle(true, id, img_iter))
        .on_press(Message::ImgMenuToggle(true, id, img_iter))
}
fn tags_list<'a>(ui: &CoverUI, id: SongId) -> iced::widget::Scrollable<'a, Message> {
    let mut row = Row::new().height(ART_ROW_H).spacing(TAG_SPACING);
    let this = &ui.state.songs[id];

    if this.new_tags.sorted.is_empty() {
        row = row.push(
            container(text("Not found").center().size(28).height(Fill).width(150)).style(filler_st),
        );
    }
    for i in 0..this.new_tags.sorted.len() {
        row = row.push(tag(ui, id, i));
    }

    scrollable(row)
        .direction(Direction::Horizontal(
            Scrollbar::new().margin(0).scroller_width(15),
        ))
        .width(Fill)
        .spacing(10)
        .style(img_scroll_st)
}
fn tag<'a>(ui: &CoverUI, id: SongId, tag_iter: ImgId) -> Button<'a, Message> {
    let this = &ui.state.songs[id];
    let tag = &this.new_tags.sorted[tag_iter];

    let label = format!("{}: {}", tag.key.to_label(), tag.value);
    let key = tag.key;
    let selected = this.selected_tags.is_select(tag.key, &tag.value);
    button(text(label).size(INNER_TEXT_SIZE))
        .style(move |theme, status| tag_st(theme, status, key, selected))
        .on_press(Message::TagToggle(id, tag_iter))
        .height(TAG_H)
}
fn orig_img<'a>(ui: &CoverUI, id: SongId, art: &ImgHandle) -> MouseArea<'a, Message> {
    let mut cont = container(image(art));
    let hovered = ui.state.songs[id].original_art_hovered;
    if hovered {
        cont = container(stack![
            cont,
            container(text("delete").size(TEXT_SIZE))
                .style(image_hover_st)
                .center(Fill),
        ]);
    }
    mouse_area(cont)
        .on_exit(Message::OrigImageHover(false, id))
        .on_enter(Message::OrigImageHover(true, id))
        .on_press(Message::RemoveImageFromFile(id))
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
