#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeRate {
    pub material: String,
    pub points_per_item: i64,
    pub min_amount: i64,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeQuote {
    pub material: String,
    pub amount: i64,
    pub points: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogItem {
    pub id: &'static str,
    pub material: &'static str,
    pub amount: i64,
    pub price: i64,
    pub category: &'static str,
}

pub const DEFAULT_SELL_RATES: &[(&str, i64, &str)] = &[
    ("COBBLESTONE", 1, "mined-blocks"),
    ("STONE", 2, "mined-blocks"),
    ("DIRT", 1, "common-blocks"),
    ("GRAVEL", 1, "common-blocks"),
    ("SAND", 1, "common-blocks"),
    ("OAK_LOG", 5, "wood"),
    ("SPRUCE_LOG", 5, "wood"),
    ("COAL", 8, "minerals"),
    ("COPPER_INGOT", 10, "minerals"),
    ("REDSTONE", 8, "minerals"),
];

pub const DEFAULT_CATALOG: &[CatalogItem] = &[
    item("block-cobblestone-64", "COBBLESTONE", 64, 96, "blocks"),
    item("block-stone-64", "STONE", 64, 192, "blocks"),
    item("block-glass-32", "GLASS", 32, 240, "blocks"),
    item("wood-oak-log-32", "OAK_LOG", 32, 240, "wood"),
    item("food-bread-16", "BREAD", 16, 160, "food"),
    item("food-cooked-beef-16", "COOKED_BEEF", 16, 420, "food"),
    item("food-golden-carrot-8", "GOLDEN_CARROT", 8, 640, "food"),
    item("utility-torch-64", "TORCH", 64, 256, "utility"),
    item("utility-arrow-64", "ARROW", 64, 384, "utility"),
    item("utility-ender-pearl-4", "ENDER_PEARL", 4, 1000, "utility"),
    item("mineral-iron-ingot-8", "IRON_INGOT", 8, 960, "minerals"),
    item("mineral-gold-ingot-8", "GOLD_INGOT", 8, 1120, "minerals"),
    item("redstone-redstone-32", "REDSTONE", 32, 384, "redstone"),
    item("redstone-repeater-8", "REPEATER", 8, 640, "redstone"),
    item("decor-name-tag-1", "NAME_TAG", 1, 1500, "utility"),
    item("transport-saddle-1", "SADDLE", 1, 1800, "utility"),
];

pub fn normalize_material(value: &str) -> Result<String, String> {
    let material = value.trim().to_ascii_uppercase().replace('-', "_");
    if material.is_empty()
        || material.len() > 64
        || !material
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    {
        return Err("invalid material".to_string());
    }
    Ok(material)
}

pub fn quote(rate: &ExchangeRate, amount: i64) -> Result<ExchangeQuote, String> {
    if !rate.enabled {
        return Err("exchange rate disabled".to_string());
    }
    if amount < rate.min_amount || amount <= 0 {
        return Err("amount below minimum".to_string());
    }
    Ok(ExchangeQuote {
        material: rate.material.clone(),
        amount,
        points: amount.saturating_mul(rate.points_per_item),
    })
}

pub fn validate_catalog(rate_lookup: impl Fn(&str) -> Option<i64>) -> Result<(), String> {
    for item in DEFAULT_CATALOG {
        if let Some(sell) = rate_lookup(item.material) {
            let sell_value = sell.saturating_mul(item.amount);
            if item.price <= sell_value {
                return Err(format!("{} can be bought for profit", item.id));
            }
        }
    }
    Ok(())
}

const fn item(
    id: &'static str,
    material: &'static str,
    amount: i64,
    price: i64,
    category: &'static str,
) -> CatalogItem {
    CatalogItem {
        id,
        material,
        amount,
        price,
        category,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cobblestone_quotes_one_point_per_block() {
        let rate = ExchangeRate {
            material: "COBBLESTONE".to_string(),
            points_per_item: 1,
            min_amount: 1,
            enabled: true,
        };
        assert_eq!(quote(&rate, 64).unwrap().points, 64);
    }

    #[test]
    fn defaults_do_not_allow_buy_sell_profit() {
        validate_catalog(|material| {
            DEFAULT_SELL_RATES
                .iter()
                .find(|rate| rate.0 == material)
                .map(|rate| rate.1)
        })
        .unwrap();
    }
}
