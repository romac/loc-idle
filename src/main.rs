mod save_system;

use iced::Font;
use loc_idle::{theme, LocIdle,save_system::load_game};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    let game_state = load_game().unwrap();
    println!("Loaded: {:?}", game_state);

    let actual_loc_idle= LocIdle::from_gamestate(&game_state);


    iced::application("LOC Idle", LocIdle::update, LocIdle::view)
        .subscription(LocIdle::subscription)
        .theme(|_| theme::night_vision())
        .default_font(Font::MONOSPACE)
        .antialiasing(false)
        .centered()
        .run_with(|| (actual_loc_idle, iced::Task::none()))
}
