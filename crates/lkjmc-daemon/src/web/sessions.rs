use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use uuid::Uuid;

const SESSION_TTL: Duration = Duration::from_secs(8 * 60 * 60);

#[derive(Clone, Default)]
pub struct WebSessions {
    inner: Arc<Mutex<BTreeMap<String, WebSession>>>,
}

#[derive(Clone)]
struct WebSession {
    token_fingerprint: String,
    csrf: String,
    expires_at: Instant,
}

impl WebSessions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_age_seconds() -> u64 {
        SESSION_TTL.as_secs()
    }

    pub fn create(&self, token: &str) -> Result<(String, String), String> {
        let session_id = Uuid::new_v4().to_string();
        let csrf = Uuid::new_v4().to_string();
        self.inner
            .lock()
            .map_err(|_| "web session lock poisoned".to_string())?
            .insert(
                session_id.clone(),
                WebSession {
                    token_fingerprint: fingerprint(token),
                    csrf: csrf.clone(),
                    expires_at: Instant::now() + SESSION_TTL,
                },
            );
        Ok((session_id, csrf))
    }

    pub fn verify(&self, session_id: &str, token: &str) -> Option<String> {
        let fingerprint = fingerprint(token);
        let mut sessions = self.inner.lock().ok()?;
        let session = sessions.get_mut(session_id)?;
        if session.expires_at <= Instant::now() {
            sessions.remove(session_id);
            return None;
        }
        if session.token_fingerprint != fingerprint {
            return None;
        }
        session.expires_at = Instant::now() + SESSION_TTL;
        Some(session.csrf.clone())
    }

    pub fn revoke(&self, session_id: &str) {
        if let Ok(mut sessions) = self.inner.lock() {
            sessions.remove(session_id);
        }
    }

    #[cfg(test)]
    pub fn expire_for_test(&self, session_id: &str) {
        if let Ok(mut sessions) = self.inner.lock() {
            if let Some(session) = sessions.get_mut(session_id) {
                session.expires_at = Instant::now() - Duration::from_secs(1);
            }
        }
    }
}

fn fingerprint(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_rotation_invalidates_sessions() -> Result<(), String> {
        let sessions = WebSessions::new();
        let (id, csrf) = sessions.create("old")?;
        assert_eq!(sessions.verify(&id, "old"), Some(csrf));
        assert_eq!(sessions.verify(&id, "new"), None);
        Ok(())
    }

    #[test]
    fn expired_sessions_are_rejected() -> Result<(), String> {
        let sessions = WebSessions::new();
        let (id, _) = sessions.create("old")?;
        sessions.expire_for_test(&id);
        assert_eq!(sessions.verify(&id, "old"), None);
        Ok(())
    }
}
