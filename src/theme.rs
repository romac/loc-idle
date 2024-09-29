use std::sync::Arc;

use iced::color;
use iced::{
    theme::{self, Palette},
    Theme,
};

pub fn night_vision() -> Theme {
    // Theme::Dracula
    Theme::Custom(Arc::new(theme::Custom::new(
        "NightVision".to_string(),
        Palette {
            background: color!(0x112119),
            text: color!(0xacecb5),
            primary: color!(0x13531c),
            success: color!(0x351866),
            danger: color!(0x661821),
        },
    )))
}
