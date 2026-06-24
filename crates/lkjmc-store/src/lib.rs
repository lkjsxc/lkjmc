#![forbid(unsafe_code)]

pub mod achievement;
pub mod audit;
pub mod command;
pub mod error;
pub mod homes;
pub mod instance;
pub mod jar;
pub mod migrate;
pub mod node;
pub mod outbox;
pub mod party;
pub mod player;
pub mod player_settings;
pub mod points;
pub mod pool;
pub mod warps;

pub const COMPONENT: &str = "lkjmc-store";

pub fn component_name() -> &'static str {
    COMPONENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_component_name() {
        assert_eq!(component_name(), "lkjmc-store");
    }
}
