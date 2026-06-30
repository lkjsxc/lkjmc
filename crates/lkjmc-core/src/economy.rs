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

mod catalog;
pub use catalog::{DEFAULT_CATALOG, DEFAULT_SELL_RATES};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cobblestone_quotes_one_point_per_block() -> Result<(), String> {
        let rate = ExchangeRate {
            material: "COBBLESTONE".to_string(),
            points_per_item: 1,
            min_amount: 1,
            enabled: true,
        };
        assert_eq!(quote(&rate, 64)?.points, 64);
        Ok(())
    }

    #[test]
    fn defaults_do_not_allow_buy_sell_profit() -> Result<(), String> {
        validate_catalog(|material| {
            DEFAULT_SELL_RATES
                .iter()
                .find(|rate| rate.0 == material)
                .map(|rate| rate.1)
        })
    }
}
