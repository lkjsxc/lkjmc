use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct SecretProvider {
    values: Arc<RwLock<Values>>,
}

#[derive(Default)]
struct Values {
    current: Option<String>,
    previous: Option<String>,
}

impl SecretProvider {
    pub fn new(value: Option<String>) -> Self {
        Self {
            values: Arc::new(RwLock::new(Values {
                current: value,
                previous: None,
            })),
        }
    }

    pub fn verify_current(&self, presented: &str) -> bool {
        self.current()
            .is_some_and(|value| constant_time_eq(presented.as_bytes(), value.as_bytes()))
    }

    pub fn configured(&self) -> bool {
        self.current().is_some_and(|value| !value.is_empty())
    }

    pub fn fingerprint(&self) -> Option<String> {
        self.current()
            .filter(|value| !value.is_empty())
            .map(|value| lkjmc_core::security::token_fingerprint(&value))
    }

    pub fn current(&self) -> Option<String> {
        self.values.read().ok()?.current.clone()
    }

    #[cfg(test)]
    pub fn previous(&self) -> Option<String> {
        self.values.read().ok()?.previous.clone()
    }

    pub fn stage(&self, value: String, previous: String) -> Result<(), String> {
        let mut values = self
            .values
            .write()
            .map_err(|_| "secret provider unavailable".to_string())?;
        values.current = Some(value);
        values.previous = Some(previous);
        Ok(())
    }

    pub fn retire_previous(&self) -> Result<(), String> {
        let mut values = self
            .values
            .write()
            .map_err(|_| "secret provider unavailable".to_string())?;
        values.previous = None;
        Ok(())
    }

    pub fn replace(&self, value: Option<String>) -> Result<(), String> {
        let mut values = self
            .values
            .write()
            .map_err(|_| "secret provider unavailable".to_string())?;
        values.current = value;
        values.previous = None;
        Ok(())
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

#[cfg(test)]
mod tests {
    use super::SecretProvider;

    #[test]
    fn unavailable_secret_has_no_value_in_its_error() {
        let provider = SecretProvider::default();
        assert!(!provider.verify_current("security-canary"));
        assert!(!provider.configured());
        assert!(provider.fingerprint().is_none());
    }
}
