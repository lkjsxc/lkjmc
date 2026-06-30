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

pub const DEFAULT_ADVENTURES: &[AdventureDefinition] = &[
    adventure(
        "end-expedition",
        "adventure.end-expedition.title",
        "adventure.end-expedition.lore",
        "challenge",
        "DRAGON_EGG",
        100,
        "end",
    ),
    adventure(
        "nether-fortress-raid",
        "adventure.nether-fortress-raid.title",
        "adventure.nether-fortress-raid.lore",
        "challenge",
        "BLAZE_ROD",
        120,
        "nether",
    ),
    adventure(
        "ancient-city-delve",
        "adventure.ancient-city-delve.title",
        "adventure.ancient-city-delve.lore",
        "stealth",
        "SCULK_SHRIEKER",
        140,
        "ancient-city",
    ),
    adventure(
        "trial-vault-run",
        "adventure.trial-vault-run.title",
        "adventure.trial-vault-run.lore",
        "combat",
        "TRIAL_KEY",
        130,
        "trial",
    ),
    adventure(
        "ocean-monument-dive",
        "adventure.ocean-monument-dive.title",
        "adventure.ocean-monument-dive.lore",
        "exploration",
        "PRISMARINE",
        110,
        "ocean",
    ),
    adventure(
        "woodland-mansion-hunt",
        "adventure.woodland-mansion-hunt.title",
        "adventure.woodland-mansion-hunt.lore",
        "exploration",
        "TOTEM_OF_UNDYING",
        150,
        "mansion",
    ),
    adventure(
        "sky-island-rush",
        "adventure.sky-island-rush.title",
        "adventure.sky-island-rush.lore",
        "timed",
        "FEATHER",
        90,
        "sky",
    ),
    adventure(
        "resource-rush",
        "adventure.resource-rush.title",
        "adventure.resource-rush.lore",
        "resource",
        "IRON_PICKAXE",
        80,
        "resource",
    ),
];

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
    fn finds_non_end_adventure() {
        let adventure = get("nether-fortress-raid");
        assert_eq!(adventure.map(|value| value.world_profile), Some("nether"));
    }
}
