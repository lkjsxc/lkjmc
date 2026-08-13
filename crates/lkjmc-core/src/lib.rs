#![forbid(unsafe_code)]

pub mod achievement;
pub mod admin;
pub mod adventure;
pub mod audit;
pub mod autosuspend;
pub mod build_info;
pub mod claim;
pub mod command;
pub mod command_registry;
mod command_shapes;
mod command_shards;
pub mod config;
pub mod data_workflow;
pub mod economy;
pub mod error;
pub mod id;
pub mod instance;
pub mod instance_create;
pub mod jar;
pub mod kubernetes;
#[cfg(test)]
mod kubernetes_tests;
pub mod model;
pub mod network_diagnostics;
pub mod network_intent;
#[cfg(test)]
mod network_intent_tests;
pub mod observability;
pub mod player;
pub mod plugin;
pub mod presence;
pub mod profile_envelope;
mod profile_limits;
#[cfg(test)]
mod profile_tests;
pub mod profile_validation;
pub mod random_teleport;
pub mod reconcile;
pub mod runtime_lifecycle;
pub mod security;
pub mod server_kind;
pub mod temporary;
pub mod validation;
pub mod wake_join;

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
