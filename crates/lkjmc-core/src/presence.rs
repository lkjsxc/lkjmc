use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerPresence {
    Unknown,
    Known { player_count: u32 },
}

impl PlayerPresence {
    pub fn empty(self) -> bool {
        matches!(self, Self::Known { player_count: 0 })
    }

    pub fn non_empty(self) -> bool {
        matches!(self, Self::Known { player_count } if player_count > 0)
    }
}
