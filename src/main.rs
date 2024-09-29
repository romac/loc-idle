use iced::Font;
use loc_idle::{theme, LocIdle};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("LOC Idle", LocIdle::update, LocIdle::view)
        .subscription(LocIdle::subscription)
        .theme(|_| theme::night_vision())
        .default_font(Font::MONOSPACE)
        .antialiasing(false)
        .centered()
        .run()
}
