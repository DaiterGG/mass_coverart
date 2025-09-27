use iced::{
    Alignment, Element,
    Length::{Fill, FillPortion},
    Theme,
    alignment::Horizontal,
    widget::{
        checkbox::Icon,
        column, container, row,
        scrollable::{Direction, Scrollbar},
    },
};

use iced::widget::scrollable;

use crate::app::{iced_app::CoverUI, song::Song, styles::*};
use iced::widget::{button, checkbox, stack, text, text_input};

pub const REGEX_LIM: usize = 7;

pub const TEXT_SIZE: f32 = 14.0;
pub const H1_SIZE: f32 = 17.0;
pub const INNER_TEXT_SIZE: f32 = 14.0;
pub const BTN_SIZE: f32 = 25.0;
pub const HEADER_H: f32 = 200.0;
pub const FILE_COL_W: f32 = 380.0;

use crate::app::iced_app::Message;

pub fn view(ui: &CoverUI) -> Element<'_, Message> {
    use Message::*;

    let theme = ui.theme();
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
        checkbox("", ui.state.parse_settings.recursive)
            .size(BTN_SIZE)
            .on_toggle(|_| RecursiveToggle)
            .style(check_st),
        h2("recursive"),
    ];
    let mut regex = row![];
    if ui.state.parse_settings.parse_file_name {
        let set = &ui.state.parse_settings;
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
                        .on_input(move |s| SeparatorInput(i, s)),
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
        if ui.state.parse_settings.reg_keys.len() > 1 {
            regex = regex.push(rem);
        }
        if ui.state.parse_settings.reg_keys.len() < REGEX_LIM {
            regex = regex.push(add);
        }
    }
    let header_color = ui.theme().extended_palette().background.weak.text;
    let files_panel = column![
        text("Open")
            .size(H1_SIZE)
            .width(Fill)
            .align_x(Alignment::Center)
            .color(header_color),
        file_button,
        folder_row,
        row![
            checkbox("", ui.state.parse_settings.parse_file_name)
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
            .size(H1_SIZE)
            .width(Fill)
            .align_x(Alignment::Center)
            .color(header_color),
        row![
            h2("downscale (height)"),
            container("").width(10),
            text_input("", &ui.state.img_settings.downscale.to_string())
                .style(input_st)
                .width(50)
                .align_x(Alignment::Center)
                .size(INNER_TEXT_SIZE)
                .on_input(DownscaleInput),
            container("").width(3),
            h2("px"),
        ],
        row![
            h2("crop to square (width)"),
            checkbox("", ui.state.img_settings.square)
                .on_toggle(|_| SquareToggle)
                .size(BTN_SIZE)
                .style(check_st),
        ]
        .spacing(10),
        row![
            h2("convert to jpg"),
            checkbox("", ui.state.img_settings.jpg)
                .on_toggle(|_| JpgToggle)
                .size(BTN_SIZE)
                .style(check_st),
        ]
        .spacing(10),
    ]
    .spacing(10);
    let header = row![
        files_panel.height(Fill).width(FillPortion(1)),
        container(container("").style(bar_st).width(1).height(Fill))
            .width(30)
            .height(Fill)
            .padding(10),
        settings_panel.width(Fill).height(Fill)
    ];
    let list = Song::generate_view_list(ui);
    let list = scrollable(list)
        .direction(Direction::Vertical(
            Scrollbar::new().margin(0).scroller_width(15),
        ))
        .width(Fill)
        .height(Fill)
        .spacing(0)
        .on_scroll(|v| Offset(v.relative_offset().y))
        .style(list_scroll_st);
    let drag_info = if ui.state.songs.is_empty() {
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
        header.height(HEADER_H).width(Fill),
        container(list).height(Fill).width(Fill),
    ]
    .height(Fill)
    .width(Fill)
    .padding(15);

    if ui.state.ui_blocked {
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
