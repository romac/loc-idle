use std::sync::Arc;
use std::time::{Duration, Instant};

use bigdecimal::BigDecimal;
use iced::theme::Palette;
use iced::widget::{button, column, container, horizontal_space, row, text, Column};
use iced::{color, theme, time, Color};
use iced::{Element, Length, Subscription, Task, Theme};

macro_rules! bd {
    ($value:expr) => {
        BigDecimal::try_from($value).unwrap()
    };
}

fn spacer() -> Element<'static, Message> {
    // container(text("")).height(iced::Length::Units(10)).into()
    text("").into()
}

fn theme() -> theme::Custom {
    let pure = Color::new(4.0 / 255.0, 17.0 / 255.0, 4.0 / 255.0, 1.0);
    let jade = Color::new(0.0 / 255.0, 17.0 / 255.0, 8.0 / 255.0, 1.0);
    let sage = Color::new(12.0 / 255.0, 17.0 / 255.0, 12.0 / 255.0, 1.0);

    theme::Custom::new(
        "NightVision".to_string(),
        Palette {
            background: sage,
            text: pure,
            primary: jade,
            success: color!(0xa6e3a1),
            danger: color!(0xf38ba8),
        },
    )
}

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("LOC Idle", LocIdle::update, LocIdle::view)
        .subscription(LocIdle::subscription)
        .theme(|_| Theme::Custom(Arc::new(theme())))
        .antialiasing(false)
        .centered()
        .run()
}

#[derive(Copy, Clone, Debug)]
struct Upgrade {
    name: &'static str,
    description: &'static str,
    enabled: fn(&LocIdle) -> bool,
    effect: fn(&mut LocIdle),
    available: bool,
}

fn upgrades() -> Vec<Upgrade> {
    vec![
        Upgrade {
            name: "Open Terminal.app",
            description: "Increase LOC/s by 1 (10 LOCs)",
            enabled: |l| l.locs >= bd!(10.0),
            effect: |l| l.coder_level += 1,
            available: true,
        },
        Upgrade {
            name: "Learn Rust",
            description: "Divide LOC/s by 2 but increase LOC cost by 3 (100 LOCs)",
            enabled: |l| l.locs >= bd!(100.0),
            effect: |l| {
                l.loc_multiplier /= 2;
                l.loc_price_multiplier *= 3;
            },
            available: true,
        },
        Upgrade {
            name: "Switch to Vim",
            description: "Multiply LOC/s by 2 (1000 LOCs)",
            enabled: |l| l.locs >= bd!(1000.0),
            effect: |l| l.loc_multiplier *= 2,
            available: true,
        },
    ]
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    WriteCode,
    HireCoder,
    Upgrade(usize),
    AiHype,
}

impl Default for LocIdle {
    fn default() -> Self {
        Self::new()
    }
}

const TICK_INTERVAL: Duration = Duration::from_millis(50);

const INITIAL_CODER_COST: f64 = 5.0;
const INITIAL_LOC_PRICE: f64 = 0.50;

const CODER_COST_INCREASE: f64 = 1.7;

const AI_HYPE_MULTIPLIER: f64 = 1.0;

#[derive(Debug)]
struct LocIdle {
    locs: BigDecimal,
    available_funds: BigDecimal,
    coders: BigDecimal,
    coder_level: BigDecimal,
    coder_cost: BigDecimal,
    ai_hype: BigDecimal,
    ai_hype_cost: BigDecimal,
    loc_price: BigDecimal,
    loc_per_sec: BigDecimal,
    loc_multiplier: BigDecimal,
    loc_price_multiplier: BigDecimal,
    upgrades: Vec<Upgrade>,

    last_time: Instant,
    delta_time: Duration,
    total_time: Duration,
}

impl LocIdle {
    fn new() -> Self {
        Self {
            locs: 0.into(),
            available_funds: 0.into(),
            coders: 0.into(),
            coder_level: 1.into(),
            coder_cost: bd!(INITIAL_CODER_COST),
            ai_hype: 0.into(),
            ai_hype_cost: 100.into(),
            loc_price: bd!(INITIAL_LOC_PRICE),
            loc_per_sec: 0.into(),
            loc_multiplier: 1.into(),
            loc_price_multiplier: 1.into(),
            upgrades: upgrades(),
            last_time: Instant::now(),
            delta_time: Duration::from_secs(0),
            total_time: Duration::from_secs(0),
        }
    }

    fn tick(&mut self, now: Instant) {
        self.delta_time = now - self.last_time;
        self.total_time += self.delta_time;
        self.last_time = now;

        self.loc_per_sec = &self.coders * (&self.coder_level - 1);
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

    fn update(&mut self, message: Message) -> Task<Message> {
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

    fn view(&self) -> Element<Message> {
        let left = column![
            //
            //
            // Header
            row![column![
                text!("Lines of Code: {}", self.locs.round(0)).size(30),
                spacer(),
                button("Write Code").on_press(Message::WriteCode),
            ]]
            .padding([20, 0]),
            //
            //
            // Business
            row![column![
                text("Business").size(30),
                text!("Available Funds: $ {}", self.available_funds.round(2)).size(20),
                text!("Price per LOC: $ {}", self.loc_price.round(2)).size(20),
                text!(
                    "Revenue per second: $ {}",
                    (&self.loc_per_sec * &self.loc_price).round(2)
                )
                .size(20),
                spacer(),
                row![
                    button("AI Hype").on_press_maybe(
                        (self.available_funds >= self.ai_hype_cost).then_some(Message::AiHype),
                    ),
                    container(text!("Level: {}", self.ai_hype.round(0)).size(20)).padding([2, 10])
                ],
                text!("Cost: $ {}", self.ai_hype_cost.round(2)).size(15)
            ]]
            .padding([20, 0]),
            //
            //
            // Development
            row![column![
                text("Development").size(30),
                text!("LOC/s: {}", &self.loc_per_sec.round(2)).size(20),
                spacer(),
                row![
                    button("Hire Coder").on_press_maybe(
                        (self.available_funds >= self.coder_cost).then_some(Message::HireCoder),
                    ),
                    container(text!("{}", self.coders.round(0)).size(20)).padding([2, 10])
                ],
                text!("Cost: $ {}", self.coder_cost.round(2)).size(15)
            ]]
            .padding([20, 0]),
        ];

        let upgrades = self
            .upgrades
            .iter()
            .enumerate()
            .filter(|(_, u)| u.available)
            .map(|(i, u)| {
                let enabled = (u.enabled)(self);
                let button = button(u.name).on_press_maybe(enabled.then_some(Message::Upgrade(i)));
                let description = text(u.description).size(15);

                Element::from(row![column![button, description]])
            });

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
                text("Upgrades").size(30),
                text!("Coder Level: {}", self.coder_level.round(0)).size(20),
                spacer(),
                Column::with_children(upgrades).spacing(20)
            ]
            .padding([20, 0]),
        ];

        container(row![
            left.width(Length::FillPortion(1)),
            right.width(Length::FillPortion(1))
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([0, 20])
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(TICK_INTERVAL).map(|_| Message::Tick)
    }
}
