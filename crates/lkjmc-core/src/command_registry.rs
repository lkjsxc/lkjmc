use std::sync::OnceLock;

use serde::Deserialize;

const SOURCE: &str = include_str!("../../../contracts/commands.json");

#[derive(Debug, Clone, Deserialize)]
struct RegistryFile {
    commands: Vec<CommandContract>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommandContract {
    pub name: String,
    pub family: String,
    pub authorization: String,
    pub surfaces: Vec<String>,
    pub doc: String,
    pub summary: String,
    pub status: String,
    pub schema_coverage: String,
    pub request_schema: String,
    pub response_schema: String,
}

static REGISTRY: OnceLock<Vec<CommandContract>> = OnceLock::new();

pub fn all() -> &'static [CommandContract] {
    REGISTRY.get_or_init(|| {
        serde_json::from_str::<RegistryFile>(SOURCE)
            .map(|file| file.commands)
            .unwrap_or_default()
    })
}

pub fn contract_for(name: &str) -> Option<&'static CommandContract> {
    all().iter().find(|entry| entry.name == name)
}

pub fn source_json() -> &'static str {
    SOURCE
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    const AUTH: &[&str] = &["admin", "open", "operator", "player"];
    const SURFACES: &[&str] = &["cli", "internal", "web"];

    #[test]
    fn registry_is_sorted_and_unique() {
        let names = all()
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
        let unique = names.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn registry_uses_known_vocabulary() {
        for entry in all() {
            assert!(
                AUTH.contains(&entry.authorization.as_str()),
                "{}",
                entry.name
            );
            assert!(!entry.surfaces.is_empty(), "{}", entry.name);
            for surface in &entry.surfaces {
                assert!(SURFACES.contains(&surface.as_str()), "{}", entry.name);
            }
        }
    }

    #[test]
    fn lookup_returns_known_command() -> Result<(), String> {
        let status = contract_for("status").ok_or_else(|| "status contract".to_string())?;
        assert_eq!(status.family, "core");
        Ok(())
    }
}
