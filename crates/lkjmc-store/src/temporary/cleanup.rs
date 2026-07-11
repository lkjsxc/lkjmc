use postgres::{GenericClient, Row};

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupCandidate {
    pub instance_id: String,
    pub lifecycle_state: String,
    pub cleanup_policy: String,
    pub world_path: String,
    pub expired: bool,
    pub cleanup_due: bool,
}

pub fn cleanup_candidates(
    client: &mut impl GenericClient,
    limit: i64,
) -> Result<Vec<CleanupCandidate>, StoreError> {
    let rows = client.query(
        "select instance_id, lifecycle_state, cleanup_policy, world_path,
         expires_at <= now(), retain_until <= now()
         from temporary_instances
         where lifecycle_state in ('created', 'starting', 'ready', 'stopped', 'failed', 'cleaning')
           and (expires_at <= now() or retain_until <= now())
         order by expires_at asc limit $1",
        &[&limit],
    )?;
    Ok(rows.into_iter().map(candidate_from_row).collect())
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn retryable_cleanup_state(state: &str) -> bool {
    matches!(
        state,
        "created" | "starting" | "ready" | "stopped" | "failed" | "cleaning"
    )
}

fn candidate_from_row(row: Row) -> CleanupCandidate {
    CleanupCandidate {
        instance_id: row.get(0),
        lifecycle_state: row.get(1),
        cleanup_policy: row.get(2),
        world_path: row.get(3),
        expired: row.get(4),
        cleanup_due: row.get(5),
    }
}

#[cfg(test)]
mod tests {
    use super::retryable_cleanup_state;

    #[test]
    fn cleanup_retry_safe() {
        assert!(retryable_cleanup_state("cleaning"));
        assert!(retryable_cleanup_state("failed"));
        assert!(!retryable_cleanup_state("cleaned"));
    }
}
