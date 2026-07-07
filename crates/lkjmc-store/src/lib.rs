#![forbid(unsafe_code)]

pub mod achievement;
pub mod admin;
pub mod admin_types;
pub mod announcement;
pub mod asset;
pub mod audit;
pub mod bootstrap;
pub mod claims;
mod claims_types;
pub mod command;
pub mod daemon_token;
pub mod daily;
pub mod discord_links;
pub mod error;
pub mod exchange;
pub mod homes;
pub mod instance;
pub mod instance_presence;
pub mod jar;
pub mod kits;
pub mod mail;
pub mod migrate;
pub mod moderation;
pub mod node;
pub mod notes;
pub mod party;
pub mod player;
pub mod player_session;
pub mod player_settings;
pub mod plugin;
pub mod points;
pub mod pool;
pub mod proxy_registration;
pub mod random_teleport;
pub mod reports;
pub mod shop;
pub mod status;
pub mod teleport;
pub mod temporary;
pub mod votes;
pub mod wake_join;
pub mod warnings;
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
