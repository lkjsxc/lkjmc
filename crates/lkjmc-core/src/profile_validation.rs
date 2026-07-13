use sha2::{Digest, Sha256};

use crate::profile_envelope::{ProfileEnvelope, PROFILE_SCHEMA};
use crate::profile_limits::{
    bounded, finite, item_valid, language, locations, namespaced, slots, unique,
};

pub struct CanonicalProfile {
    pub envelope: ProfileEnvelope,
    pub json: Vec<u8>,
    pub sha256: String,
}

pub fn canonical_profile(input: &[u8]) -> Result<CanonicalProfile, String> {
    if input.len() > 1_048_576 {
        return Err("profile exceeds 1 MiB".into());
    }
    let mut profile: ProfileEnvelope =
        serde_json::from_slice(input).map_err(|error| format!("invalid typed profile: {error}"))?;
    validate(&profile)?;
    profile.inventory.sort_by_key(|entry| entry.slot);
    profile.armor.sort_by_key(|entry| entry.slot);
    profile.ender_chest.sort_by_key(|entry| entry.slot);
    profile.potion_effects.sort_by(|a, b| a.id.cmp(&b.id));
    profile.plugin_data.sort_by(|a, b| a.key.cmp(&b.key));
    profile.homes.sort_by(|a, b| a.name.cmp(&b.name));
    profile.warps.sort_by(|a, b| a.name.cmp(&b.name));
    profile.achievements.sort();
    for slot in profile
        .inventory
        .iter_mut()
        .chain(profile.armor.iter_mut())
        .chain(profile.ender_chest.iter_mut())
    {
        slot.item.enchantments.sort_by(|a, b| a.id.cmp(&b.id));
    }
    if let Some(item) = profile.offhand.as_mut() {
        item.enchantments.sort_by(|a, b| a.id.cmp(&b.id));
    }
    let json = serde_json::to_vec(&profile).map_err(|error| error.to_string())?;
    let sha256 = format!("{:x}", Sha256::digest(&json));
    Ok(CanonicalProfile {
        envelope: profile,
        json,
        sha256,
    })
}

fn validate(profile: &ProfileEnvelope) -> Result<(), String> {
    if profile.schema != PROFILE_SCHEMA {
        return Err("profile schema must be lkjmc-profile-one".into());
    }
    slots(&profile.inventory, 41, "inventory")?;
    slots(&profile.armor, 4, "armor")?;
    slots(&profile.ender_chest, 27, "enderChest")?;
    if profile.selected_hotbar_slot > 8 {
        return Err("selectedHotbarSlot exceeds 8".into());
    }
    if let Some(item) = &profile.offhand {
        item_valid(item)?;
    }
    finite(profile.experience.progress, "experience.progress")?;
    finite(profile.vitals.health, "vitals.health")?;
    finite(profile.vitals.saturation, "vitals.saturation")?;
    if !(0.0..=1.0).contains(&profile.experience.progress)
        || profile.vitals.health < 0.0
        || profile.vitals.health > 2048.0
        || profile.vitals.food > 20
        || profile.vitals.saturation < 0.0
        || profile.vitals.saturation > 20.0
        || profile.vitals.air < -20
        || profile.vitals.air > 30_000
    {
        return Err("profile vitals or experience out of bounds".into());
    }
    unique(
        profile.potion_effects.iter().map(|v| v.id.as_str()),
        "potion effect",
    )?;
    for effect in &profile.potion_effects {
        namespaced(&effect.id)?;
    }
    unique(
        profile.plugin_data.iter().map(|v| v.key.as_str()),
        "plugin key",
    )?;
    for value in &profile.plugin_data {
        namespaced(&value.key)?;
        bounded(&value.value, 8192, "plugin value")?;
    }
    locations(&profile.homes, "home")?;
    locations(&profile.warps, "warp")?;
    unique(
        profile.achievements.iter().map(String::as_str),
        "achievement",
    )?;
    for id in &profile.achievements {
        namespaced(id)?;
    }
    bounded(&profile.settings.privacy, 64, "privacy")?;
    language(&profile.language)
}
