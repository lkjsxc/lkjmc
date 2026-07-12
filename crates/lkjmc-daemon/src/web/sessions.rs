use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

const SESSION_TTL: Duration = Duration::from_secs(8 * 60 * 60);
const MAX_SESSIONS: usize = 256;
const LOGIN_WINDOW: Duration = Duration::from_secs(60);
const MAX_LOGIN_ATTEMPTS: usize = 8;
const MAX_LOGIN_SOURCES: usize = 32;

#[derive(Clone)]
pub struct WebSessions {
    inner: Arc<Mutex<Store>>,
}

struct Store {
    key: String,
    sessions: BTreeMap<String, WebSession>,
    attempts: BTreeMap<String, LoginAttempts>,
}

#[derive(Clone)]
struct WebSession {
    token_fingerprint: String,
    expires_at: Instant,
}

struct LoginAttempts {
    started_at: Instant,
    count: usize,
}

impl WebSessions {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Store {
                key: Uuid::new_v4().to_string(),
                sessions: BTreeMap::new(),
                attempts: BTreeMap::new(),
            })),
        }
    }

    pub fn max_age_seconds() -> u64 {
        SESSION_TTL.as_secs()
    }

    pub fn allow_login(&self, source: &str) -> bool {
        let Ok(mut store) = self.inner.lock() else {
            return false;
        };
        let now = Instant::now();
        store
            .attempts
            .retain(|_, attempts| now.duration_since(attempts.started_at) < LOGIN_WINDOW);
        if !store.attempts.contains_key(source) && store.attempts.len() >= MAX_LOGIN_SOURCES {
            return false;
        }
        let attempts = store
            .attempts
            .entry(source.to_string())
            .or_insert(LoginAttempts {
                started_at: now,
                count: 0,
            });
        if attempts.count >= MAX_LOGIN_ATTEMPTS {
            return false;
        }
        attempts.count += 1;
        true
    }

    pub fn login_succeeded(&self, source: &str) {
        if let Ok(mut store) = self.inner.lock() {
            store.attempts.remove(source);
        }
    }

    pub fn create(&self, token_fingerprint: &str) -> Result<(String, String), String> {
        let id = Uuid::new_v4().to_string();
        let mut store = self
            .inner
            .lock()
            .map_err(|_| "web session store unavailable".to_string())?;
        store
            .sessions
            .retain(|_, session| session.expires_at > Instant::now());
        if store.sessions.len() >= MAX_SESSIONS {
            return Err("web session capacity reached".into());
        }
        store.sessions.insert(
            id.clone(),
            WebSession {
                token_fingerprint: token_fingerprint.into(),
                expires_at: Instant::now() + SESSION_TTL,
            },
        );
        Ok((id.clone(), csrf(&store.key, &id)))
    }

    pub fn verify(&self, id: &str, token_fingerprint: &str) -> Option<String> {
        let mut store = self.inner.lock().ok()?;
        let session = store.sessions.get_mut(id)?;
        if session.expires_at <= Instant::now() || session.token_fingerprint != token_fingerprint {
            store.sessions.remove(id);
            return None;
        }
        session.expires_at = Instant::now() + SESSION_TTL;
        Some(csrf(&store.key, id))
    }

    pub fn revoke(&self, id: &str) {
        if let Ok(mut store) = self.inner.lock() {
            store.sessions.remove(id);
        }
    }

    #[cfg(test)]
    pub fn expire_for_test(&self, id: &str) {
        if let Ok(mut store) = self.inner.lock() {
            if let Some(session) = store.sessions.get_mut(id) {
                session.expires_at = Instant::now() - Duration::from_secs(1);
            }
        }
    }
}

fn csrf(key: &str, id: &str) -> String {
    hex(&Sha256::digest(format!("{key}:{id}").as_bytes()))
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_limiter_denies_the_ninth_attempt() {
        let sessions = WebSessions::new();
        assert!((0..MAX_LOGIN_ATTEMPTS).all(|_| sessions.allow_login("source")));
        assert!(!sessions.allow_login("source"));
    }

    #[test]
    fn changed_fingerprint_invalidates_sessions() -> Result<(), String> {
        let sessions = WebSessions::new();
        let (id, csrf) = sessions.create("old-fingerprint")?;
        assert_eq!(sessions.verify(&id, "old-fingerprint"), Some(csrf));
        assert_eq!(sessions.verify(&id, "new-fingerprint"), None);
        Ok(())
    }
}
