use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RandomTeleportPolicy {
    pub cost_points: i64,
    pub cooldown_seconds: i64,
    pub min_radius: i32,
    pub max_radius: i32,
    pub max_attempts: u32,
    pub allowed_worlds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RandomTeleportDecision {
    Allowed,
    Disabled(String),
    Cooldown { remaining_seconds: i64 },
}

impl RandomTeleportPolicy {
    pub fn defaults() -> Self {
        Self {
            cost_points: 250,
            cooldown_seconds: 600,
            min_radius: 750,
            max_radius: 5000,
            max_attempts: 64,
            allowed_worlds: vec!["*".to_string()],
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.cost_points < 0 {
            return Err("cost_points must be non-negative".to_string());
        }
        if self.cooldown_seconds < 0 {
            return Err("cooldown_seconds must be non-negative".to_string());
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
}

#[cfg(test)]
mod tests {
    use super::{RandomTeleportDecision, RandomTeleportPolicy};

    #[test]
    fn defaults_are_valid_and_visible() {
        let policy = RandomTeleportPolicy::defaults();
        assert_eq!(policy.cost_points, 250);
        assert_eq!(policy.cooldown_seconds, 600);
        assert_eq!(policy.min_radius, 750);
        assert_eq!(policy.max_radius, 5000);
        assert_eq!(policy.max_attempts, 64);
        assert!(policy.validate().is_ok());
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
