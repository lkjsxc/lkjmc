use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopItem {
    pub id: String,
    pub title_key: String,
    pub price_points: i64,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purchase {
    pub item: ShopItem,
    pub duplicate: bool,
    pub refundable: bool,
}

mod catalog;
mod settlement;

pub use catalog::{
    canonical_adventure_metadata, get_item, is_canonical_adventure_delivery, list_items,
    record_purchase, seed_default_catalog, upsert_item, upsert_item_with_metadata,
    valid_minecraft_item, validate_delivery_metadata,
};
pub use settlement::{purchase, refund_purchase, replay};
