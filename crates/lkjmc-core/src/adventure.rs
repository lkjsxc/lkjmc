#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureDefinition {
    pub id: &'static str,
    pub title_key: &'static str,
    pub lore_key: &'static str,
    pub category: &'static str,
    pub icon_material: &'static str,
    pub price_points: i64,
    pub min_party_size: u8,
    pub max_party_size: u8,
    pub max_lifetime_seconds: u32,
    pub retention_seconds: u32,
    pub runtime_kind: &'static str,
    pub jar_kind: &'static str,
    pub world_profile: &'static str,
    pub cleanup_policy: &'static str,
    pub permission: &'static str,
    pub enabled: bool,
}

pub const DEFAULT_ADVENTURES: &[AdventureDefinition] = &[adventure(
    "end-expedition",
    "adventure.end-expedition.title",
    "adventure.end-expedition.lore",
    "challenge",
    "DRAGON_EGG",
    100,
    "end",
)];

pub fn get(id: &str) -> Option<&'static AdventureDefinition> {
    DEFAULT_ADVENTURES
        .iter()
        .find(|adventure| adventure.id == id)
}

pub fn validate(definitions: &[AdventureDefinition]) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    for adventure in definitions {
        if !ids.insert(adventure.id) {
            return Err(format!("duplicate adventure id: {}", adventure.id));
        }
        if adventure.price_points <= 0 {
            return Err(format!("{} price must be positive", adventure.id));
        }
        if adventure.min_party_size == 0 || adventure.min_party_size > adventure.max_party_size {
            return Err(format!("{} party bounds are invalid", adventure.id));
        }
        if adventure.max_lifetime_seconds == 0 || adventure.retention_seconds == 0 {
            return Err(format!("{} lifetime must be positive", adventure.id));
        }
    }
    Ok(())
}

const fn adventure(
    id: &'static str,
    title_key: &'static str,
    lore_key: &'static str,
    category: &'static str,
    icon_material: &'static str,
    price_points: i64,
    world_profile: &'static str,
) -> AdventureDefinition {
    AdventureDefinition {
        id,
        title_key,
        lore_key,
        category,
        icon_material,
        price_points,
        min_party_size: 1,
        max_party_size: 4,
        max_lifetime_seconds: 3600,
        retention_seconds: 600,
        runtime_kind: "folia",
        jar_kind: "folia",
        world_profile,
        cleanup_policy: "delete",
        permission: "lkjmc.user.adventure",
        enabled: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_is_valid() {
        assert!(validate(DEFAULT_ADVENTURES).is_ok());
    }

    #[test]
    fn generic_adventures_are_withdrawn() {
        assert!(get("resource-rush").is_none());
    }
}
