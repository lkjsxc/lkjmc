use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdminRole {
    Owner,
    Operator,
    Moderator,
    Support,
    Builder,
    Player,
}

impl AdminRole {
    pub fn id(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Operator => "operator",
            Self::Moderator => "moderator",
            Self::Support => "support",
            Self::Builder => "builder",
            Self::Player => "player",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Owner,
            Self::Operator,
            Self::Moderator,
            Self::Support,
            Self::Builder,
            Self::Player,
        ]
    }

    pub fn permissions(self) -> &'static [&'static str] {
        match self {
            Self::Owner => OWNER,
            Self::Operator => OPERATOR,
            Self::Moderator => MODERATOR,
            Self::Support => SUPPORT,
            Self::Builder => BUILDER,
            Self::Player => PLAYER,
        }
    }
}

const OWNER: &[&str] = &[
    "lkjmc.admin.admin",
    "lkjmc.admin.economy",
    "lkjmc.admin.status",
    "lkjmc.admin.reload",
    "lkjmc.admin.instance.list",
    "lkjmc.admin.instance.create",
    "lkjmc.admin.instance.start",
    "lkjmc.admin.instance.stop",
    "lkjmc.admin.instance.restart",
    "lkjmc.admin.instance.delete",
    "lkjmc.admin.send",
    "lkjmc.admin.warp",
    "lkjmc.admin.claim",
    "lkjmc.admin.reports",
    "lkjmc.admin.warn",
    "lkjmc.admin.ban",
    "lkjmc.admin.mute",
];
const OPERATOR: &[&str] = &[
    "lkjmc.admin.status",
    "lkjmc.admin.reload",
    "lkjmc.admin.instance.list",
    "lkjmc.admin.instance.create",
    "lkjmc.admin.instance.start",
    "lkjmc.admin.instance.stop",
    "lkjmc.admin.instance.restart",
    "lkjmc.admin.reports",
];
const MODERATOR: &[&str] = &[
    "lkjmc.admin.status",
    "lkjmc.admin.reports",
    "lkjmc.admin.warn",
    "lkjmc.admin.ban",
    "lkjmc.admin.mute",
    "lkjmc.admin.claim",
];
const SUPPORT: &[&str] = &["lkjmc.admin.status", "lkjmc.admin.reports"];
const BUILDER: &[&str] = &[
    "lkjmc.admin.status",
    "lkjmc.admin.warp",
    "lkjmc.admin.claim",
];
const PLAYER: &[&str] = &[];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_expands_to_dangerous_permissions() {
        assert!(AdminRole::Owner
            .permissions()
            .contains(&"lkjmc.admin.instance.delete"));
        assert!(AdminRole::Owner
            .permissions()
            .contains(&"lkjmc.admin.admin"));
    }

    #[test]
    fn support_cannot_mutate_instances() {
        assert!(!AdminRole::Support
            .permissions()
            .contains(&"lkjmc.admin.instance.start"));
    }
}
