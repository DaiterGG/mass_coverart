use iced::{
    Border, Color, Theme, border,
    widget::{
        button::{self},
        checkbox::{self},
        container,
        scrollable::{self, Rail, Scroller},
        text_input,
    },
};

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
        placeholder: palette.background.weak.color,
    }
}

pub fn filler_st(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();

    container::Style {
        background: Some(p.background.base.color.into()),
        text_color: Some(p.background.weak.color),
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

pub fn list_bg_st(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();

    container::Style {
        background: Some(p.background.strong.color.into()),
        border: Border {
            width: 0.0,
            radius: 10.0.into(),
            color: p.danger.base.color,
        },
        text_color: Some(p.background.weak.color),
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
        _ => p.background.base.color,
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
