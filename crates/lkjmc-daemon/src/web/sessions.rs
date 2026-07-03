use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct WebSessions {
    inner: Arc<Mutex<BTreeMap<String, WebSession>>>,
}

#[derive(Clone)]
struct WebSession {
    token_fingerprint: String,
    csrf: String,
}

impl WebSessions {
    pub fn new() -> Self {
        Self::default()
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
                },
            );
        Ok((session_id, csrf))
    }

    pub fn verify(&self, session_id: &str, token: &str) -> Option<String> {
        let fingerprint = fingerprint(token);
        self.inner.lock().ok()?.get(session_id).and_then(|session| {
            (session.token_fingerprint == fingerprint).then(|| session.csrf.clone())
        })
    }

    pub fn revoke(&self, session_id: &str) {
        if let Ok(mut sessions) = self.inner.lock() {
            sessions.remove(session_id);
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
}
