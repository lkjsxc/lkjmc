#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementDefinition {
    pub id: &'static str,
    pub category: &'static str,
    pub title_key: &'static str,
    pub description_key: &'static str,
    pub icon_material: &'static str,
    pub criteria_kind: &'static str,
    pub threshold: i64,
    pub reward_points: i64,
    pub hidden: bool,
    pub repeatable: bool,
}

mod catalog;
pub use catalog::DEFAULT_ACHIEVEMENTS;

pub fn by_criteria(kind: &str) -> Vec<&'static AchievementDefinition> {
    DEFAULT_ACHIEVEMENTS
        .iter()
        .filter(|item| item.criteria_kind == kind)
        .collect()
}

pub fn validate(definitions: &[AchievementDefinition]) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    for item in definitions {
        if !ids.insert(item.id) {
            return Err(format!("duplicate achievement id: {}", item.id));
        }
        if item.threshold <= 0 {
            return Err(format!("{} threshold must be positive", item.id));
        }
        if item.reward_points < 0 {
            return Err(format!("{} reward must not be negative", item.id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_valid() {
        assert!(validate(DEFAULT_ACHIEVEMENTS).is_ok());
    }
    #[test]
    fn shop_purchase_has_progress_definition() {
        assert!(!by_criteria("shop-purchase").is_empty());
    }
}
