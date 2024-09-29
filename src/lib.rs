use std::time::{Duration, Instant};

use bigdecimal::BigDecimal;
use iced::time;
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, Column};
use iced::{Element, Length, Subscription, Task};

mod macros;
pub mod theme;
pub mod upgrade;
pub mod widget;

use crate::upgrade::Upgrade;
use crate::widget::{header, hspace, upgrade_button, vspace};

const TICK_INTERVAL: Duration = Duration::from_millis(50);

const INITIAL_CODER_COST: f64 = 5.0;
const INITIAL_LOC_PRICE: f64 = 0.50;
const CODER_COST_INCREASE: f64 = 1.7;
const AI_HYPE_MULTIPLIER: f64 = 1.0;

const FONT_BASE: f32 = 18.0;
const FONT_MAIN: f32 = FONT_BASE * 1.5;
const FONT_SMALL: f32 = FONT_BASE - 3.0;
const FONT_HEADER: f32 = FONT_BASE + 3.0;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    WriteCode,
    HireCoder,
    Upgrade(usize),
    AiHype,
}

#[derive(Debug)]
pub struct LocIdle {
    locs: BigDecimal,
    available_funds: BigDecimal,
    coders: BigDecimal,
    coder_level: BigDecimal,
    coder_cost: BigDecimal,
    ai_hype: BigDecimal,
    ai_hype_cost: BigDecimal,
    loc_price: BigDecimal,
    loc_per_sec: BigDecimal,
    loc_per_sec_base: BigDecimal,
    loc_multiplier: BigDecimal,
    loc_price_multiplier: BigDecimal,
    upgrades: Vec<Upgrade>,

    last_time: Instant,
    delta_time: Duration,
    total_time: Duration,
}

impl Default for LocIdle {
    fn default() -> Self {
        Self::new()
    }
}

impl LocIdle {
    pub fn new() -> Self {
        Self {
            locs: 0.into(),
            available_funds: 0.into(),
            coders: 0.into(),
            coder_level: 0.into(),
            coder_cost: bd!(INITIAL_CODER_COST),
            ai_hype: 0.into(),
            ai_hype_cost: 100.into(),
            loc_price: bd!(INITIAL_LOC_PRICE),
            loc_per_sec: 0.into(),
            loc_per_sec_base: 0.into(),
            loc_multiplier: 1.into(),
            loc_price_multiplier: 1.into(),
            upgrades: upgrade::all(),
            last_time: Instant::now(),
            delta_time: Duration::from_secs(0),
            total_time: Duration::from_secs(0),
        }
    }

    fn tick(&mut self, now: Instant) {
        self.delta_time = now - self.last_time;
        self.total_time += self.delta_time;
        self.last_time = now;

        let line_per_coder = &self.loc_per_sec_base * &self.coder_level;
        self.loc_per_sec = &self.coders * line_per_coder;
        self.loc_per_sec *= &self.loc_multiplier;

        self.loc_price = bd!(INITIAL_LOC_PRICE);
        self.loc_price += bd!(AI_HYPE_MULTIPLIER) * &self.ai_hype;
        self.loc_price *= &self.loc_price_multiplier;

        let loc_delta = self.loc_delta();
        self.locs += &loc_delta;
        self.available_funds += loc_delta * &self.loc_price;
    }

    fn loc_delta(&self) -> BigDecimal {
        &self.loc_per_sec * bd!(self.delta_time.as_secs_f64())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.tick(Instant::now());
            }

            Message::WriteCode => {
                self.locs += 1;
                self.available_funds += &self.loc_price;
            }

            Message::HireCoder => {
                if self.available_funds >= self.coder_cost {
                    self.available_funds -= &self.coder_cost;
                    self.coders += 1;
                    self.coder_cost *= bd!(CODER_COST_INCREASE);
                }
            }

            Message::Upgrade(index) => {
                if self.upgrades[index].available && (self.upgrades[index].enabled)(self) {
                    (self.upgrades[index].effect)(self);
                    self.upgrades[index].available = false;
                }
            }

            Message::AiHype => {
                if self.available_funds >= self.ai_hype_cost {
                    self.available_funds -= &self.ai_hype_cost;
                    self.ai_hype += 1;
                    self.ai_hype_cost *= 2;
                }
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let left = column![
            //
            //
            // Header
            row![column![
                text!("Lines of Code: {}", self.locs.round(0)).size(FONT_MAIN),
                vspace(10),
                button("Write Code").on_press(Message::WriteCode),
            ]]
            .padding([20, 0]),
            //
            //
            // Business
            row![column![
                header("Business"),
                text!("Available Funds:    $ {}", self.available_funds.round(2)).size(FONT_BASE),
                text!("Price per LOC:      $ {}", self.loc_price.round(2)).size(FONT_BASE),
                text!(
                    "Revenue per second: $ {}",
                    (&self.loc_per_sec * &self.loc_price).round(2)
                )
                .size(FONT_BASE),
                vspace(20),
                row![
                    button("AI Hype").on_press_maybe(
                        (self.available_funds >= self.ai_hype_cost).then_some(Message::AiHype),
                    ),
                    container(text!("Level: {}", self.ai_hype.round(0)).size(FONT_BASE))
                        .padding([2, 10])
                ],
                vspace(5),
                text!("Cost: $ {}", self.ai_hype_cost.round(2)).size(FONT_SMALL)
            ]]
            .padding([20, 0]),
            //
            //
            // Development
            row![column![
                header("Development"),
                text!("LOC/s: {}", &self.loc_per_sec.round(2)).size(FONT_BASE),
                vspace(20),
                row![
                    button("Hire Coder").on_press_maybe(
                        (self.available_funds >= self.coder_cost).then_some(Message::HireCoder),
                    ),
                    container(text!("{}", self.coders.round(0)).size(FONT_BASE)).padding([2, 10])
                ],
                vspace(5),
                text!("Cost: $ {}", self.coder_cost.round(2)).size(FONT_SMALL)
            ]]
            .padding([20, 0]),
        ];

        let upgrades = self
            .upgrades
            .iter()
            .enumerate()
            .filter(|(_, u)| u.available)
            .map(|(i, u)| upgrade_button(self, i, u));

        let right = column![
            // FPS
            row![
                horizontal_space(),
                text!("{:.0} FPS", 1.0 / self.delta_time.as_secs_f64()).size(10)
            ]
            .padding([10, 0]),
            //
            //
            // Upgrades
            column![
                header("Upgrades"),
                text!("Coder Level: {}", self.coder_level.round(0)).size(FONT_BASE),
                vspace(20),
                scrollable(row![
                    Column::with_children(upgrades).spacing(FONT_BASE),
                    hspace(20)
                ])
            ]
            .padding([20, 0]),
        ];

        container(
            row![
                left.width(Length::FillPortion(1)),
                right.width(Length::FillPortion(1))
            ]
            .spacing(100),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([0, 20])
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(TICK_INTERVAL).map(|_| Message::Tick)
    }
}
