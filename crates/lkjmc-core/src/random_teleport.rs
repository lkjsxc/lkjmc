use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RandomTeleportPolicy {
    pub profile_id: String,
    pub target_environment: String,
    pub cost_points: i64,
    pub cooldown_seconds: i64,
    pub min_radius: i32,
    pub max_radius: i32,
    pub max_attempts: u32,
    pub allowed_worlds: Vec<String>,
    pub confirmation_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RandomTeleportDecision {
    Allowed,
    Disabled(String),
    Cooldown { remaining_seconds: i64 },
}

impl RandomTeleportPolicy {
    pub fn defaults() -> Self {
        Self::overworld()
    }

    pub fn profile(profile_id: &str) -> Option<Self> {
        match profile_id {
            "" | "overworld" => Some(Self::overworld()),
            "nether" => Some(Self::paid("nether", "nether", 500)),
            "end" => Some(Self::paid("end", "the_end", 750)),
            _ => None,
        }
    }

    pub fn profiles() -> Vec<Self> {
        ["overworld", "nether", "end"]
            .into_iter()
            .filter_map(Self::profile)
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.profile_id.is_empty() {
            return Err("profile_id is required".to_string());
        }
        if self.cost_points < 0 || self.cooldown_seconds < 0 {
            return Err("cost and cooldown must be non-negative".to_string());
        }
        if self.min_radius < 0 || self.max_radius < self.min_radius {
            return Err("radius range is invalid".to_string());
        }
        if self.max_attempts == 0 {
            return Err("max_attempts must be positive".to_string());
        }
        if self.allowed_worlds.is_empty() {
            return Err("at least one world is required".to_string());
        }
        Ok(())
    }

    pub fn decide(&self, cooldown_remaining_seconds: i64) -> RandomTeleportDecision {
        if let Err(error) = self.validate() {
            return RandomTeleportDecision::Disabled(error);
        }
        if cooldown_remaining_seconds > 0 {
            return RandomTeleportDecision::Cooldown {
                remaining_seconds: cooldown_remaining_seconds,
            };
        }
        RandomTeleportDecision::Allowed
    }

    pub fn world_allowed(&self, world: &str) -> bool {
        self.allowed_worlds
            .iter()
            .any(|allowed| allowed == "*" || allowed == world)
    }

    fn overworld() -> Self {
        Self {
            profile_id: "overworld".to_string(),
            target_environment: "normal".to_string(),
            cost_points: 0,
            cooldown_seconds: 600,
            min_radius: 750,
            max_radius: 5000,
            max_attempts: 64,
            allowed_worlds: vec!["*".to_string()],
            confirmation_required: false,
        }
    }

    fn paid(profile_id: &str, target_environment: &str, cost_points: i64) -> Self {
        Self {
            profile_id: profile_id.to_string(),
            target_environment: target_environment.to_string(),
            cost_points,
            cooldown_seconds: 600,
            min_radius: 750,
            max_radius: 5000,
            max_attempts: 64,
            allowed_worlds: vec!["*".to_string()],
            confirmation_required: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RandomTeleportDecision, RandomTeleportPolicy};

    #[test]
    fn overworld_default_is_free_and_unconfirmed() {
        let policy = RandomTeleportPolicy::defaults();
        assert_eq!(policy.profile_id, "overworld");
        assert_eq!(policy.cost_points, 0);
        assert!(!policy.confirmation_required);
        assert_eq!(policy.target_environment, "normal");
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn dimension_profiles_are_paid_and_confirmed() {
        let nether =
            RandomTeleportPolicy::profile("nether").unwrap_or_else(RandomTeleportPolicy::defaults);
        let end =
            RandomTeleportPolicy::profile("end").unwrap_or_else(RandomTeleportPolicy::defaults);
        assert!(nether.cost_points > 0 && nether.confirmation_required);
        assert!(end.cost_points > nether.cost_points && end.confirmation_required);
        assert_eq!(nether.target_environment, "nether");
        assert_eq!(end.target_environment, "the_end");
    }

    #[test]
    fn rejects_invalid_radius_and_attempts() {
        let mut policy = RandomTeleportPolicy::defaults();
        policy.max_radius = 1;
        assert!(policy.validate().is_err());
        policy = RandomTeleportPolicy::defaults();
        policy.max_attempts = 0;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn reports_cooldown_before_allowing() {
        let policy = RandomTeleportPolicy::defaults();
        assert_eq!(
            policy.decide(42),
            RandomTeleportDecision::Cooldown {
                remaining_seconds: 42
            }
        );
        assert_eq!(policy.decide(0), RandomTeleportDecision::Allowed);
    }
}
