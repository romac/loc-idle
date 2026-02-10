use std::time::{Duration, Instant};

use bigdecimal::BigDecimal;
use bigdecimal::num_traits::ToPrimitive;

use iced::time;
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, Column};
use iced::{Element, Length, Subscription, Task};

mod macros;
pub mod theme;
pub mod upgrade;
pub mod widget;
pub mod save_system;

use crate::upgrade::Upgrade;
use crate::widget::{header, hspace, upgrade_button, vspace};
use crate::save_system::{GameState,close_game};

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
    SaveAndQuit(iced::window::Id),
    IgnoreWindowEvent,
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

    /// I really don't understand this field, it may have no utilities with save states
    /// Consider deleting this attribute if you don't see the use of it
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

            Message::SaveAndQuit(window_id)  => {
            println!("1: SaveAndQuit handler called");
            println!("2: Current state - LOCs: {}, Coders: {}", self.locs, self.coders);
            
            // For now, just test without actual saving
            println!("3: Would save here...");
            let result=close_game(self);
            
            println!("4: Closing window {:?}",result);
            iced::window::close::<iced::window::Id>(window_id.into());
            }

            Message::IgnoreWindowEvent=>{
                // Nothing done here for now, but can be usefull for later use
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
            .filter(|(_, u)| u.available && (u.enabled)(self))
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
        let tick_subscription = time::every(TICK_INTERVAL).map(|_| Message::Tick);
        
        let window_subscription = iced::window::events().map(|(id, event)| {
            match event {
                iced::window::Event::CloseRequested => {
                    println!("Window close requested! Window ID: {:?}", id);
                    Message::SaveAndQuit(id)
                }
                iced::window::Event::Closed =>
                {
                    println!("Window closed! Window ID: {:?}", id);
                    Message::SaveAndQuit(id)
                }
                _ => Message::IgnoreWindowEvent
            }
        });
        
        Subscription::batch([tick_subscription, window_subscription])
    }

    pub fn to_gamestate(&self) -> GameState {
            GameState {
                locs: self.locs.clone(),
                available_funds: self.available_funds.clone(),
                coders: self.coders.clone(),
                coder_level:self.coder_level.clone(),
                ai_hype: self.ai_hype.clone(),
                loc_per_sec: self.loc_per_sec.clone(),
                loc_per_sec_base:self.loc_per_sec_base.clone(),
                loc_multiplier: self.loc_multiplier.clone(),
                loc_price_multiplier: self.loc_price_multiplier.clone(),
                upgrades: self.upgrades.iter()
                    .enumerate()
                    .filter(|(_, u)| !u.available)
                    .map(|(i, _)| i)
                    .collect(),
                total_time: self.total_time.as_secs(),
            }
        }
    
    pub fn from_gamestate(gamestate:&GameState) -> LocIdle{

        /// Helper function: pow function via loop:<br>
        /// Because I'm not even capable of doing a O(1) solution
        /// with the pow function and needs to use a O(n) function
        /// so this is a technical debt for the end game with 
        /// especially huge numbers
        fn pow_loop(base: BigDecimal, exponent: u64) -> BigDecimal {
            if exponent == 0 {
                return BigDecimal::from(1);
            }
            let mut result = base.clone();
            for _ in 1..exponent {
                result = &result * &base;
            }
            result
        }


        // Compute of coder_cost
        let coders_u64 = gamestate.coders
            .with_scale(0)
            .round(0)
            .to_u64()
            .unwrap_or(0);
        

        let coder_cost = if coders_u64 == 0 {
            bd!(INITIAL_CODER_COST)
        } else {
            bd!(INITIAL_CODER_COST) * pow_loop(bd!(CODER_COST_INCREASE), coders_u64)
        };        
                

        // Compute of ai_hype_cost
        let ai_hype_u64 = gamestate.ai_hype
            .with_scale(0)
            .round(0)
            .to_u64()
            .unwrap_or(0);

        let ai_hype_cost = if ai_hype_u64 == 0 {
            bd!(100)
        } else {
            bd!(100) * pow_loop(bd!(2), ai_hype_u64)
        };
        

        // Compute of loc_price
        let loc_price = (bd!(INITIAL_LOC_PRICE) + bd!(AI_HYPE_MULTIPLIER) * gamestate.ai_hype.clone())
         * gamestate.loc_price_multiplier.clone();

        
        // Create and mark upgrades
        let mut upgrades = upgrade::all();
        for &index in &gamestate.upgrades {
            if index < upgrades.len() {
                upgrades[index].available = false;
            }
        }

        
        LocIdle { 
            locs: gamestate.locs.clone(), 
            available_funds: gamestate.available_funds.clone(), 
            coders: gamestate.coders.clone(),
            coder_level: gamestate.coder_level.clone(),
            coder_cost: coder_cost, 
            ai_hype: gamestate.ai_hype.clone(), 
            ai_hype_cost: ai_hype_cost.into(), 
            loc_price: loc_price, 
            loc_per_sec: gamestate.loc_per_sec.clone(), 
            loc_per_sec_base: gamestate.loc_per_sec_base.clone(), 
            loc_multiplier: gamestate.loc_multiplier.clone(), 
            loc_price_multiplier: gamestate.loc_price_multiplier.clone(), 
            upgrades: upgrades, 
            last_time: Instant::now(),
            delta_time: Duration::from_secs(0),
            total_time: Duration::from_secs(gamestate.total_time)
        }
    }
    
}
