use iced::font::Weight;
use iced::widget::{button, column, container, horizontal_rule, text, Space};
use iced::{Alignment, Element, Font, Length, Padding};

use crate::upgrade::Upgrade;
use crate::{LocIdle, Message, FONT_BASE, FONT_HEADER, FONT_SMALL};

pub fn vspace(height: impl Into<Length>) -> Element<'static, Message> {
    Space::with_height(height).into()
}

pub fn hspace(width: impl Into<Length>) -> Element<'static, Message> {
    Space::with_width(width).into()
}

pub fn header(title: &str) -> Element<'static, Message> {
    column![
        text(title.to_string()).size(FONT_HEADER),
        horizontal_rule(2),
        vspace(10),
    ]
    .into()
}

pub fn upgrade_button(l: &LocIdle, i: usize, u: &Upgrade) -> Element<'static, Message> {
    let mut bold = Font::DEFAULT;
    bold.weight = Weight::Bold;

    let enabled = (u.enabled)(l);
    let title = text(u.name).font(bold).size(FONT_BASE);
    let description = text(u.description).size(FONT_SMALL);
    let required = container(text(u.required).size(FONT_SMALL))
        .padding(Padding::ZERO.top(5))
        .width(Length::Fill)
        .align_x(Alignment::End);

    let panel = column![title, description, required].width(Length::Fill);

    button(panel)
        .on_press_maybe(enabled.then_some(Message::Upgrade(i)))
        .into()
}
