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
    get_item, list_items, record_purchase, seed_default_catalog, upsert_item,
    upsert_item_with_metadata, valid_minecraft_item,
};
pub use settlement::{purchase, refund_purchase, replay};
