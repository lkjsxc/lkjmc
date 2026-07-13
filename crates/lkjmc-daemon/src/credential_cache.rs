use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use lkjmc_store::daemon_token::DaemonTokenRecord;
use lkjmc_store::error::StoreError;

const MAX_CREDENTIALS: usize = 128;

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
        client: &mut postgres::Client,
        credential: &str,
    ) -> Result<Option<DaemonTokenRecord>, StoreError> {
        let hash = lkjmc_core::security::token_hash(credential);
        let mut transaction = client.transaction().map_err(StoreError::from)?;
        let revision = lkjmc_store::daemon_token::lock_current_revision(&mut transaction)?;
        let cached = self.cached(&hash, revision)?;
        let cache_hit = cached.is_some();
        let record = match cached {
            Some(record) => Some(record),
            None => lkjmc_store::daemon_token::find_active(&mut transaction, &hash)?,
        };
        transaction.commit().map_err(StoreError::from)?;
        if !cache_hit {
            if record.is_some() {
                lkjmc_store::daemon_token::touch_active(client, &hash)?;
            }
            if let Some(record) = record.as_ref() {
                self.store(hash, revision, record.clone())?;
            }
        }
        Ok(record)
    }

    fn cached(&self, hash: &str, revision: i64) -> Result<Option<DaemonTokenRecord>, StoreError> {
        let mut state = self.state.lock().map_err(cache_unavailable)?;
        reset_revision(&mut state, revision);
        expire(&mut state);
        Ok(state.entries.get(hash).cloned())
    }

    fn store(
        &self,
        hash: String,
        revision: i64,
        record: DaemonTokenRecord,
    ) -> Result<(), StoreError> {
        let mut state = self.state.lock().map_err(cache_unavailable)?;
        reset_revision(&mut state, revision);
        expire(&mut state);
        if !state.entries.contains_key(&hash) {
            evict_for(&mut state.entries, &hash);
        }
        state.entries.insert(hash, record);
        Ok(())
    }
}

fn cache_unavailable<T>(_: T) -> StoreError {
    StoreError::invalid_state("credential cache unavailable")
}

fn reset_revision(state: &mut CacheState, revision: i64) {
    if state.revision != Some(revision) {
        state.entries.clear();
        state.revision = Some(revision);
    }
}

fn expire(state: &mut CacheState) {
    expire_at(&mut state.entries, unix_micros());
}

fn expire_at(entries: &mut BTreeMap<String, DaemonTokenRecord>, now: i64) {
    entries.retain(|_, entry| entry.expires_at_micros > now);
}

fn evict_for(entries: &mut BTreeMap<String, DaemonTokenRecord>, hash: &str) {
    if entries.len() >= MAX_CREDENTIALS {
        if let Some(key) = entries.keys().next().cloned() {
            entries.remove(&key);
        }
    }
    debug_assert!(entries.len() < MAX_CREDENTIALS || entries.contains_key(hash));
}

fn unix_micros() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_micros()).ok())
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "credential_cache_tests.rs"]
mod tests;
