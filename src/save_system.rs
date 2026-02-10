use std::env;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::LocIdle;

#[derive(Deserialize,Serialize,Debug)]
pub struct GameState{
    pub locs: BigDecimal,
    pub available_funds:BigDecimal,
    pub coders: BigDecimal,
    pub coder_level: BigDecimal,
    pub ai_hype:BigDecimal,
    pub loc_per_sec: BigDecimal,
    pub loc_per_sec_base: BigDecimal,
    pub loc_multiplier: BigDecimal,
    pub loc_price_multiplier:BigDecimal,
    pub upgrades:Vec<usize>,
    pub total_time:u64,
}

pub fn load_game() -> Result<GameState, Box<dyn std::error::Error>> {
    // Where cargo run was executed
    let project_root = env::current_dir()?;
    let save_path = project_root.join("data/save.json");
    
    match std::fs::read_to_string(save_path) {
        Ok(json_text) => {
            let state: GameState = serde_json::from_str(&json_text)?;
            Ok(state)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Create default if no file
            Ok(GameState {
                locs: 0.into(),
                available_funds:0.into(),
                coders: 0.into(),
                coder_level:0.into(),
                ai_hype:0.into(),
                loc_per_sec:0.into(),
                loc_per_sec_base:0.into(),
                loc_multiplier:1.into(),
                loc_price_multiplier:1.into(),
                upgrades:Vec::new(),
                total_time:0
            })
        }
        Err(e) => Err(e.into()),
    }
}

pub fn close_game(locidle:&LocIdle) -> Result<(), Box<dyn std::error::Error>> {
    let project_root = env::current_dir()?;
    let save_path = project_root.join("data/save.json");
    
    // Create directory if it doesn't exist
    if let Some(parent) = save_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Serialize to JSON string
    let json_text = serde_json::to_string(&locidle.to_gamestate())?;
    
    // Write to file
    std::fs::write(save_path, json_text)?;
    
    Ok(())
}
