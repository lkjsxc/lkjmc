use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

const SESSION_TTL: Duration = Duration::from_secs(8 * 60 * 60);
const MAX_SESSIONS: usize = 256;
#[derive(Clone)]
pub struct WebSessions {
    inner: Arc<Mutex<Store>>,
}
struct Store {
    key: String,
    sessions: BTreeMap<String, WebSession>,
}
#[derive(Clone)]
struct WebSession {
    token_fingerprint: String,
    expires_at: Instant,
}
impl WebSessions {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Store {
                key: Uuid::new_v4().to_string(),
                sessions: BTreeMap::new(),
            })),
        }
    }
    pub fn max_age_seconds() -> u64 {
        SESSION_TTL.as_secs()
    }
    pub fn create(&self, token: &str) -> Result<(String, String), String> {
        let id = Uuid::new_v4().to_string();
        let mut store = self
            .inner
            .lock()
            .map_err(|_| "web session lock poisoned".to_string())?;
        store
            .sessions
            .retain(|_, session| session.expires_at > Instant::now());
        if store.sessions.len() >= MAX_SESSIONS {
            return Err("web session capacity reached".into());
        }
        store.sessions.insert(
            id.clone(),
            WebSession {
                token_fingerprint: fingerprint(token),
                expires_at: Instant::now() + SESSION_TTL,
            },
        );
        Ok((id.clone(), csrf(&store.key, &id)))
    }
    pub fn verify(&self, id: &str, token: &str) -> Option<String> {
        let mut store = self.inner.lock().ok()?;
        let session = store.sessions.get_mut(id)?;
        if session.expires_at <= Instant::now() || session.token_fingerprint != fingerprint(token) {
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
fn fingerprint(value: &str) -> String {
    hex(&Sha256::digest(value.as_bytes()))
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
    fn token_rotation_invalidates_sessions() -> Result<(), String> {
        let sessions = WebSessions::new();
        let (id, csrf) = sessions.create("old")?;
        assert_eq!(sessions.verify(&id, "old"), Some(csrf));
        assert_eq!(sessions.verify(&id, "new"), None);
        Ok(())
    }
}
