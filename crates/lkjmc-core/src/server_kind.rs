use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerImplementation {
    Velocity,
    Paper,
    Folia,
    Purpur,
    VanillaCustom,
    ModdedCustom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub region_scheduler: bool,
    pub paper_api: bool,
    pub purpur_config: bool,
    pub velocity_forwarding: bool,
}

impl ServerImplementation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Velocity => "velocity",
            Self::Paper => "paper",
            Self::Folia => "folia",
            Self::Purpur => "purpur",
            Self::VanillaCustom => "vanilla-custom",
            Self::ModdedCustom => "modded-custom",
        }
    }

    pub fn capabilities(self) -> ServerCapabilities {
        match self {
            Self::Velocity => ServerCapabilities::new(false, false, false, false),
            Self::Paper => ServerCapabilities::new(false, true, false, true),
            Self::Folia => ServerCapabilities::new(true, true, false, true),
            Self::Purpur => ServerCapabilities::new(false, true, true, true),
            Self::VanillaCustom | Self::ModdedCustom => {
                ServerCapabilities::new(false, false, false, false)
            }
        }
    }
}

impl ServerCapabilities {
    pub const fn new(
        region_scheduler: bool,
        paper_api: bool,
        purpur_config: bool,
        velocity_forwarding: bool,
    ) -> Self {
        Self {
            region_scheduler,
            paper_api,
            purpur_config,
            velocity_forwarding,
        }
    }
}

impl FromStr for ServerImplementation {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "velocity" => Ok(Self::Velocity),
            "paper" => Ok(Self::Paper),
            "folia" => Ok(Self::Folia),
            "purpur" => Ok(Self::Purpur),
            "vanilla-custom" => Ok(Self::VanillaCustom),
            "modded-custom" => Ok(Self::ModdedCustom),
            _ => Err(format!("unsupported server implementation: {value}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_server_implementation_strings() {
        assert_eq!("paper".parse(), Ok(ServerImplementation::Paper));
        assert_eq!("folia".parse(), Ok(ServerImplementation::Folia));
        assert_eq!("purpur".parse(), Ok(ServerImplementation::Purpur));
        assert_eq!("velocity".parse(), Ok(ServerImplementation::Velocity));
    }

    #[test]
    fn rejects_invalid_server_implementation_strings() {
        assert!("spigot".parse::<ServerImplementation>().is_err());
    }

    #[test]
    fn exposes_runtime_capabilities() {
        assert!(ServerImplementation::Folia.capabilities().region_scheduler);
        assert!(ServerImplementation::Purpur.capabilities().purpur_config);
        assert!(!ServerImplementation::Purpur.capabilities().region_scheduler);
    }
}
