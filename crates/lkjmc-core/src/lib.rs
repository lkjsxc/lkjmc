#![forbid(unsafe_code)]

pub mod admin;
pub mod audit;
pub mod autosuspend;
pub mod bootstrap;
pub mod claim;
pub mod command;
pub mod config;
pub mod economy;
pub mod error;
pub mod id;
pub mod instance;
pub mod jar;
pub mod model;
pub mod network_diagnostics;
pub mod player;
pub mod presence;
pub mod reconcile;
pub mod security;
pub mod server_kind;
pub mod temporary;
pub mod validation;

pub const COMPONENT: &str = "lkjmc-core";

pub fn component_name() -> &'static str {
    COMPONENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_component_name() {
        assert_eq!(component_name(), "lkjmc-core");
    }
}
