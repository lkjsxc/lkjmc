use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use lkjmc_store::daemon_token::DaemonTokenRecord;
use postgres::GenericClient;

#[derive(Clone, Default)]
pub struct CredentialCache {
    state: Arc<Mutex<CacheState>>,
}

#[derive(Default)]
struct CacheState {
    revision: Option<i64>,
    entries: BTreeMap<String, DaemonTokenRecord>,
}

impl CredentialCache {
    pub fn authenticate(
        &self,
        client: &mut impl GenericClient,
        credential: &str,
    ) -> Result<Option<DaemonTokenRecord>, ()> {
        let revision = lkjmc_store::daemon_token::current_revision(client).map_err(|_| ())?;
        let hash = lkjmc_core::security::token_hash(credential);
        if let Some(record) = self.cached(&hash, revision)? {
            return Ok(Some(record));
        }
        let record = lkjmc_store::daemon_token::find_active(client, &hash).map_err(|_| ())?;
        if let Some(record) = record.as_ref() {
            self.store(hash, revision, record.clone())?;
        }
        Ok(record)
    }

    fn cached(&self, hash: &str, revision: i64) -> Result<Option<DaemonTokenRecord>, ()> {
        let mut state = self.state.lock().map_err(|_| ())?;
        if state.revision != Some(revision) {
            state.entries.clear();
            state.revision = Some(revision);
        }
        let now = unix_seconds();
        state
            .entries
            .retain(|_, entry| entry.expires_at_seconds > now);
        Ok(state.entries.get(hash).cloned())
    }

    fn store(&self, hash: String, revision: i64, record: DaemonTokenRecord) -> Result<(), ()> {
        let mut state = self.state.lock().map_err(|_| ())?;
        if state.revision != Some(revision) {
            state.entries.clear();
            state.revision = Some(revision);
        }
        state.entries.insert(hash, record);
        Ok(())
    }
}

fn unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn revision_change_drops_cached_credential() -> Result<(), String> {
        let cache = CredentialCache::default();
        let record = DaemonTokenRecord {
            credential_id: Uuid::nil(),
            surface: "web".into(),
            principal_kind: "operator".into(),
            principal_id: "operator-1".into(),
            scopes: vec!["lkjmc.admin.admin".into()],
            expires_at_seconds: unix_seconds() + 60,
        };
        cache
            .store("hash".into(), 1, record)
            .map_err(|_| "cache store failed".to_string())?;
        assert!(cache
            .cached("hash", 1)
            .map_err(|_| "cache read failed".to_string())?
            .is_some());
        assert!(cache
            .cached("hash", 2)
            .map_err(|_| "cache read failed".to_string())?
            .is_none());
        Ok(())
    }
}
