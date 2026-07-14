use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginId {
    LkjmcPaper,
    LkjmcVelocity,
    #[serde(rename = "viaversion")]
    ViaVersion,
    #[serde(rename = "viabackwards")]
    ViaBackwards,
    Geyser,
    Floodgate,
}

impl PluginId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LkjmcPaper => "lkjmc-paper",
            Self::LkjmcVelocity => "lkjmc-velocity",
            Self::ViaVersion => "viaversion",
            Self::ViaBackwards => "viabackwards",
            Self::Geyser => "geyser",
            Self::Floodgate => "floodgate",
        }
    }
}
