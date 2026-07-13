use std::collections::HashSet;

use crate::profile_envelope::{ProfileItem, ProfileSlot, SavedLocation};

pub(crate) fn slots(values: &[ProfileSlot], limit: u8, field: &str) -> Result<(), String> {
    if values.len() > usize::from(limit) {
        return Err(format!("too many {field} slots"));
    }
    let mut seen = HashSet::new();
    for value in values {
        if value.slot >= limit || !seen.insert(value.slot) {
            return Err(format!("invalid or duplicate {field} slot"));
        }
        item_valid(&value.item)?;
    }
    Ok(())
}

pub(crate) fn item_valid(item: &ProfileItem) -> Result<(), String> {
    namespaced(&item.material)?;
    if item.amount == 0 || item.amount > 127 || item.lore.len() > 64 || item.enchantments.len() > 64
    {
        return Err("item values out of bounds".into());
    }
    if let Some(name) = &item.custom_name {
        bounded(name, 1024, "custom name")?;
    }
    for line in &item.lore {
        bounded(line, 1024, "lore")?;
    }
    unique(
        item.enchantments.iter().map(|v| v.id.as_str()),
        "enchantment",
    )?;
    for value in &item.enchantments {
        namespaced(&value.id)?;
        if value.level == 0 || value.level > 255 {
            return Err("enchantment level out of bounds".into());
        }
    }
    Ok(())
}

pub(crate) fn locations(values: &[SavedLocation], field: &str) -> Result<(), String> {
    if values.len() > 128 {
        return Err(format!("too many {field}s"));
    }
    unique(values.iter().map(|v| v.name.as_str()), field)?;
    for value in values {
        bounded(&value.name, 64, field)?;
        bounded(&value.server, 128, "server")?;
        namespaced(&value.world)?;
        for (number, name) in [(value.x, "x"), (value.y, "y"), (value.z, "z")] {
            if !number.is_finite() || number.abs() > 30_000_000.0 {
                return Err(format!("invalid location {name}"));
            }
        }
        finite(value.yaw, "yaw")?;
        finite(value.pitch, "pitch")?;
    }
    Ok(())
}

pub(crate) fn namespaced(value: &str) -> Result<(), String> {
    let Some((namespace, path)) = value.split_once(':') else {
        return Err("identifier is not namespaced".into());
    };
    let valid = |text: &str| {
        !text.is_empty()
            && text.len() <= 128
            && text
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"._-/".contains(&c))
    };
    if !valid(namespace) || !valid(path) {
        return Err("invalid namespaced identifier".into());
    }
    Ok(())
}

pub(crate) fn language(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
    {
        return Err("invalid language".into());
    }
    Ok(())
}

pub(crate) fn unique<'a>(values: impl Iterator<Item = &'a str>, field: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    if values.into_iter().any(|value| !seen.insert(value)) {
        Err(format!("duplicate {field}"))
    } else {
        Ok(())
    }
}
pub(crate) fn bounded(value: &str, max: usize, field: &str) -> Result<(), String> {
    if value.len() > max {
        Err(format!("{field} too long"))
    } else {
        Ok(())
    }
}
pub(crate) fn finite(value: impl Into<f64>, field: &str) -> Result<(), String> {
    if value.into().is_finite() {
        Ok(())
    } else {
        Err(format!("{field} is not finite"))
    }
}
