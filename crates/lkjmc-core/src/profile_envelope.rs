use serde::{Deserialize, Serialize};

pub const PROFILE_SCHEMA: &str = "lkjmc-profile-one";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProfileEnvelope {
    pub schema: String,
    pub inventory: Vec<ProfileSlot>,
    pub armor: Vec<ProfileSlot>,
    pub offhand: Option<ProfileItem>,
    pub selected_hotbar_slot: u8,
    pub ender_chest: Vec<ProfileSlot>,
    pub experience: Experience,
    pub vitals: Vitals,
    pub potion_effects: Vec<PotionEffect>,
    pub game_mode: Option<GameMode>,
    pub plugin_data: Vec<PluginDatum>,
    pub homes: Vec<SavedLocation>,
    pub warps: Vec<SavedLocation>,
    pub points: i64,
    pub achievements: Vec<String>,
    pub settings: ProfileSettings,
    pub language: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProfileSlot {
    pub slot: u8,
    pub item: ProfileItem,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProfileItem {
    pub material: String,
    pub amount: u8,
    pub damage: u32,
    pub custom_name: Option<String>,
    pub lore: Vec<String>,
    pub enchantments: Vec<Enchantment>,
    pub custom_model_data: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Enchantment {
    pub id: String,
    pub level: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Experience {
    pub progress: f32,
    pub level: u32,
    pub total: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Vitals {
    pub health: f32,
    pub food: u8,
    pub saturation: f32,
    pub air: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PotionEffect {
    pub id: String,
    pub amplifier: u8,
    pub duration_ticks: u32,
    pub ambient: bool,
    pub particles: bool,
    pub icon: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PluginDatum {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SavedLocation {
    pub name: String,
    pub server: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProfileSettings {
    pub menu_enabled: bool,
    pub hud_enabled: bool,
    pub tips_enabled: bool,
    pub privacy: String,
}
