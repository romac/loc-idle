use crate::{bd, LocIdle};

#[derive(Copy, Clone, Debug)]
pub struct Upgrade {
    pub name: &'static str,
    pub description: &'static str,
    pub required: &'static str,
    pub enabled: fn(&LocIdle) -> bool,
    pub effect: fn(&mut LocIdle),
    pub available: bool,
}

pub fn all() -> Vec<Upgrade> {
    vec![
        Upgrade {
            name: "Open nano",
            description: "Start writing code",
            required: "10 LOCs",
            enabled: |l| l.locs >= bd!(10.0),
            effect: |l| {
                l.loc_per_sec_base += 1;
                l.coder_level += 1;
            },
            available: true,
        },
        Upgrade {
            name: "Drink Coffee",
            description: "Increase LOC/s by 5",
            required: "20 LOCs",
            enabled: |l| l.locs >= bd!(20.0),
            effect: |l| {
                l.loc_per_sec_base += 5;
            },
            available: true,
        },
        Upgrade {
            name: "Learn Rust",
            description: "Divide LOC/s by 2 but increase LOC cost by 3",
            required: "100 LOCs",
            enabled: |l| l.locs >= bd!(100.0),
            effect: |l| {
                l.coder_level += 1;
                l.loc_multiplier /= 2;
                l.loc_price_multiplier *= 3;
            },
            available: true,
        },
        Upgrade {
            name: "Switch to Vim",
            description: "Multiply LOC/s by 2",
            required: "1000 LOCs",
            enabled: |l| l.locs >= bd!(1000.0),
            effect: |l| {
                l.coder_level += 1;
                l.loc_multiplier *= 2;
            },
            available: true,
        },
    ]
}
