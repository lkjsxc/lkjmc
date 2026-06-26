pub mod decision;
pub mod desired;
pub mod diagnostic;
pub mod effect;
pub mod facts;
pub mod plan;
pub mod plugin;
pub mod ports;

pub use decision::plan_bootstrap;
pub use desired::*;
pub use diagnostic::*;
pub use effect::*;
pub use facts::*;
pub use plan::*;
pub use plugin::*;

use serde::{Deserialize, Serialize};

use crate::config::{BedrockEntry, JavaEntry};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapRequest {
    pub profile: BootstrapProfile,
    pub accept_minecraft_eula: bool,
    pub java_entry: JavaEntry,
    pub bedrock_entry: BedrockEntry,
    pub plugin_policy: PluginPolicy,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BootstrapProfile {
    Playable,
}

#[cfg(test)]
mod tests;
