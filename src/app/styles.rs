use iced::{
    Border, Color, Theme, border,
    theme::{
        Custom, Palette,
        palette::{self, Danger, Extended, Pair, Primary, Secondary, Success, Warning},
    },
    widget::{
        button::{self},
        checkbox::{self},
        container,
        scrollable::{self, Rail, Scroller},
        text_input,
    },
};

pub fn miasma_theme() -> Theme {
    let primary = Color::from_rgb8(120, 130, 74);
    let secondary = Color::from_rgb8(187, 119, 68);
    let bg = Color::from_rgb8(34, 34, 34);
    let bg_strong = Color::from_rgb8(28, 28, 28);
    let bg_strong_text = Color::from_rgb8(90, 90, 90);
    let bg_weak = Color::from_rgb8(46, 46, 46);
    let bg_weak_text = Color::from_rgb8(60, 60, 60);
    let text = Color::from_rgb8(215, 196, 131);
    let success = Color::from_rgb8(95, 135, 95);
    let danger = Color::from_rgb8(104, 87, 66);
    let warning = Color::from_rgb8(104, 87, 66);
    Theme::custom_with_fn(
        "miasma".to_string(),
        Palette {
            text,
            primary,
            success,
            danger,
            background: bg,
            warning,
        },
        |_| Extended {
            primary: Primary::generate(primary, bg, text),
            background: palette::Background {
                weak: Pair {
                    color: bg_weak,
                    text: bg_weak_text,
                },
                base: Pair::new(bg, text),
                strong: Pair {
                    color: bg_strong,
                    text: bg_strong_text,
                },
                weakest: Pair::new(bg, text),
                neutral: Pair::new(bg, text),
                weaker: Pair::new(bg, text),
                stronger: Pair::new(bg, text),
                strongest: Pair::new(bg, text),
            },

            warning: Warning::generate(warning, bg, text),
            secondary: Secondary {
                base: Pair::new(secondary, text),
                weak: Pair::new(secondary, text),
                strong: Pair::new(secondary, text),
            },
            success: Success::generate(success, bg, text),
            danger: Danger::generate(danger, bg, text),
            is_dark: true,
        },
    )
}

pub fn preview_close_st(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(Color::from_rgba8(0, 0, 0, 0.7).into()),
        ..button::Style::default()
    }
}
pub fn button_st(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let color = match status {
        button::Status::Hovered => palette.secondary.base.color,
        _ => palette.danger.base.color,
    };
    button::Style {
        background: Some(palette.background.base.color.into()),
        border: Border {
            width: 1.0,
            radius: 10.0.into(),
            color,
        },
        text_color: palette.background.base.text,
        ..button::Style::default()
    }
}
pub fn check_st(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let palette = theme.extended_palette();

    let color = match status {
        checkbox::Status::Hovered { .. } => palette.secondary.base.color,
        _ => palette.danger.base.color,
    };
    checkbox::Style {
        background: palette.background.base.color.into(),
        border: Border {
            width: 1.0,
            radius: 10.0.into(),
            color,
        },
        icon_color: palette.secondary.base.color,
        text_color: None,
    }
}

pub fn bar_st(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        ..container::Style::default()
    }
}
pub fn input_st(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.extended_palette();

    text_input::Style {
        background: palette.background.base.color.into(),
        border: Border {
            width: 1.0,
            radius: 10.0.into(),
            color: palette.danger.base.color,
        },
        selection: palette.primary.base.color,
        icon: palette.secondary.base.color,
        value: palette.background.base.text,
        placeholder: palette.background.strong.text,
    }
}

pub fn filler_st(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();

    container::Style {
        background: Some(p.background.base.color.into()),
        text_color: Some(p.background.weak.text),
        ..container::Style::default()
    }
}

pub fn add_remove(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let color = match status {
        button::Status::Hovered => palette.secondary.base.color,
        _ => palette.danger.base.color,
    };

    button::Style {
        background: Some(palette.background.base.color.into()),
        border: Border {
            width: 1.0,
            radius: 100.0.into(),
            color,
        },
        text_color: palette.background.base.text,
        ..button::Style::default()
    }
}

pub fn image_hover_st(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();

    let mut bg = p.background.strong.color;
    bg.a = 0.6;
    container::Style {
        background: Some(bg.into()),
        border: Border {
            width: 0.0,
            radius: 0.0.into(),
            color: p.danger.base.color,
        },
        text_color: Some(p.background.base.text),
        ..container::Style::default()
    }
}
pub fn select_menu_st(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();

    container::Style {
        background: Some(p.background.strong.color.into()),
        border: Border {
            width: 1.0,
            radius: 10.0.into(),
            color: p.danger.base.color,
        },
        text_color: Some(p.background.base.text),
        ..container::Style::default()
    }
}
pub fn preview_box_st(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();

    container::Style {
        background: Some(p.background.strong.color.into()),
        border: Border {
            width: 1.0,
            radius: 10.0.into(),
            color: p.danger.base.color,
        },
        text_color: Some(p.background.weak.text),
        ..container::Style::default()
    }
}
pub fn list_bg_st(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();

    container::Style {
        background: Some(p.background.strong.color.into()),
        border: Border {
            width: 0.0,
            radius: 10.0.into(),
            color: p.danger.base.color,
        },
        text_color: Some(p.background.weak.text),
        ..container::Style::default()
    }
}
pub fn list_scroll_st(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let p = theme.extended_palette();
    let bg = match status {
        scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered,
            ..
        } if is_vertical_scrollbar_hovered => p.primary.weak.color,
        scrollable::Status::Dragged {
            is_vertical_scrollbar_dragged,
            ..
        } if is_vertical_scrollbar_dragged => p.primary.base.color,
        _ => p.background.strong.color,
    };
    let rail = Rail {
        background: None,
        border: Border {
            width: 0.0,
            radius: 0.0.into(),
            color: p.danger.base.color,
        },
        scroller: Scroller {
            color: bg,
            border: Border {
                width: 1.0,
                radius: 10.0.into(),
                color: p.background.weak.color,
            },
        },
    };
    scrollable::Style {
        container: container::Style {
            background: None,
            border: Border {
                width: 0.0,
                radius: 10.0.into(),
                color: p.background.base.color,
            },
            ..container::Style::default()
        },
        gap: None,
        vertical_rail: rail,
        horizontal_rail: rail,
    }
}

pub fn img_scroll_st(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let p = theme.extended_palette();
    let bg = match status {
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            ..
        } if is_horizontal_scrollbar_hovered => p.primary.weak.color,
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            ..
        } if is_horizontal_scrollbar_dragged => p.primary.base.color,
        _ => p.background.strong.color,
    };
    let rail = Rail {
        background: None,
        border: Border {
            width: 0.0,
            radius: 0.0.into(),
            color: p.danger.base.color,
        },
        scroller: Scroller {
            color: bg,
            border: Border {
                width: 1.0,
                radius: 10.0.into(),
                color: p.background.weak.color,
            },
        },
    };
    scrollable::Style {
        container: container::Style {
            background: None,
            border: Border {
                width: 0.0,
                radius: 10.0.into(),
                color: p.background.base.color,
            },
            ..container::Style::default()
        },
        gap: None,
        vertical_rail: rail,
        horizontal_rail: rail,
    }
}
pub fn blank_button(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();

    button::Style {
        background: None,
        border: Border {
            width: 0.0,
            radius: 0.0.into(),
            color: p.primary.base.color,
        },
        ..button::Style::default()
    }
}
pub fn scroll_bar_st(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();

    let bg = match status {
        button::Status::Hovered => p.primary.weak.color,
        button::Status::Pressed => p.primary.base.color,
        _ => p.background.base.color,
    };
    button::Style {
        background: Some(bg.into()),
        border: Border {
            width: 1.0,
            radius: 10.0.into(),
            color: p.background.weak.color,
        },
        ..button::Style::default()
    }
}
pub fn image_selected_st(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();

    container::Style {
        background: Some(p.background.strong.color.into()),
        border: Border {
            width: 3.0,
            radius: 10.0.into(),
            color: p.primary.base.color,
        },
        text_color: Some(p.background.base.text),
        ..container::Style::default()
    }
}
pub fn list_border_st(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();

    container::Style {
        background: None,
        border: Border {
            width: 1.0,
            radius: 10.0.into(),
            color: p.background.weak.color,
        },
        ..container::Style::default()
    }
}
pub fn item_cont_st(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.base.color.into()),
        border: Border {
            width: 1.0,
            radius: 10.0.into(),
            color: palette.background.weak.color,
        },
        ..container::Style::default()
    }
}
