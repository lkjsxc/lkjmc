use super::AchievementDefinition;

macro_rules! achievement {
    ($id:literal, $category:literal, $icon:literal, $kind:literal, $threshold:literal, $reward:literal) => {
        AchievementDefinition {
            id: $id,
            category: $category,
            title_key: concat!("achievement.", $id, ".title"),
            description_key: concat!("achievement.", $id, ".description"),
            icon_material: $icon,
            criteria_kind: $kind,
            threshold: $threshold,
            reward_points: $reward,
            hidden: false,
            repeatable: false,
        }
    };
}

macro_rules! adventure {
    ($id:literal) => {
        AchievementDefinition {
            id: $id,
            category: "adventure",
            title_key: concat!("achievement.", $id, ".title"),
            description_key: concat!("achievement.", $id, ".description"),
            icon_material: "DRAGON_EGG",
            criteria_kind: "adventure-complete",
            threshold: 1,
            reward_points: 100,
            hidden: false,
            repeatable: false,
        }
    };
}

pub const DEFAULT_ACHIEVEMENTS: &[AchievementDefinition] = &[
    achievement!(
        "first-login",
        "welcome",
        "PLAYER_HEAD",
        "first-login",
        1,
        25
    ),
    achievement!("first-home", "settlement", "RED_BED", "home-set", 1, 25),
    achievement!(
        "first-claim",
        "settlement",
        "GOLDEN_SHOVEL",
        "claim-created",
        1,
        40
    ),
    achievement!(
        "first-shop-purchase",
        "economy",
        "EMERALD",
        "shop-purchase",
        1,
        25
    ),
    achievement!(
        "first-exchange",
        "economy",
        "COBBLESTONE",
        "exchange-commit",
        1,
        25
    ),
    achievement!("first-kit", "economy", "CHEST", "kit-claim", 1, 20),
    achievement!("first-vote", "community", "SUNFLOWER", "vote-reward", 1, 30),
    achievement!("first-mail", "social", "WRITABLE_BOOK", "mail-send", 1, 20),
    achievement!(
        "first-party",
        "social",
        "NAME_TAG",
        "party-create-or-join",
        1,
        20
    ),
    achievement!("daily-streak-3", "routine", "CLOCK", "daily-streak", 3, 50),
    achievement!("daily-streak-7", "routine", "CLOCK", "daily-streak", 7, 125),
    achievement!(
        "daily-streak-30",
        "routine",
        "CLOCK",
        "daily-streak",
        30,
        750
    ),
    achievement!(
        "miner-1000",
        "craft",
        "IRON_PICKAXE",
        "block-exchange-total",
        1000,
        100
    ),
    achievement!("traveler-25", "exploration", "COMPASS", "warp-use", 25, 75),
    achievement!(
        "safe-return",
        "adventure",
        "ENDER_PEARL",
        "adventure-return",
        1,
        50
    ),
    adventure!("end-expedition"),
    adventure!("nether-fortress-raid"),
    adventure!("ancient-city-delve"),
    adventure!("trial-vault-run"),
    adventure!("ocean-monument-dive"),
    adventure!("woodland-mansion-hunt"),
    adventure!("sky-island-rush"),
    adventure!("resource-rush"),
];
