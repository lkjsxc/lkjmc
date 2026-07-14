use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

const BUCKETS_MS: [u64; 6] = [1, 5, 25, 100, 500, 2_000];
pub(crate) const SERIES_CAP: usize = 64;

#[derive(Clone, Default)]
pub(crate) struct Metrics {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    requests_ok: AtomicU64,
    requests_error: AtomicU64,
    admission_rejected: AtomicU64,
    latency: [AtomicU64; 7],
    database_error: AtomicU64,
    runtime_error: AtomicU64,
    sync_error: AtomicU64,
    jvm_error: AtomicU64,
    bundles_ok: AtomicU64,
    bundles_error: AtomicU64,
}

impl Metrics {
    pub(crate) fn request(&self, ok: bool, elapsed: Duration) {
        (if ok {
            &self.inner.requests_ok
        } else {
            &self.inner.requests_error
        })
        .fetch_add(1, Ordering::Relaxed);
        let millis = elapsed.as_millis().min(u128::from(u64::MAX)) as u64;
        let index = BUCKETS_MS
            .iter()
            .position(|bound| millis <= *bound)
            .unwrap_or(6);
        for value in self.inner.latency.iter().skip(index) {
            value.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(crate) fn admission_rejected(&self) {
        self.inner
            .admission_rejected
            .fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn database_error(&self) {
        self.inner.database_error.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn bundle(&self, ok: bool) {
        (if ok {
            &self.inner.bundles_ok
        } else {
            &self.inner.bundles_error
        })
        .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn export(&self, in_flight: usize) -> String {
        let mut lines = vec![
            line(
                "lkjmc_requests_total",
                "outcome=\"succeeded\"",
                self.inner.requests_ok.load(Ordering::Relaxed),
            ),
            line(
                "lkjmc_requests_total",
                "outcome=\"failed\"",
                self.inner.requests_error.load(Ordering::Relaxed),
            ),
            line(
                "lkjmc_admission_rejected_total",
                "reason=\"capacity\"",
                self.inner.admission_rejected.load(Ordering::Relaxed),
            ),
            line(
                "lkjmc_admission_in_flight",
                "component=\"daemon\"",
                in_flight as u64,
            ),
            line(
                "lkjmc_database_errors_total",
                "class=\"database\"",
                self.inner.database_error.load(Ordering::Relaxed),
            ),
            line(
                "lkjmc_runtime_errors_total",
                "class=\"runtime\"",
                self.inner.runtime_error.load(Ordering::Relaxed),
            ),
            line(
                "lkjmc_sync_errors_total",
                "class=\"sync\"",
                self.inner.sync_error.load(Ordering::Relaxed),
            ),
            line(
                "lkjmc_jvm_errors_total",
                "class=\"jvm\"",
                self.inner.jvm_error.load(Ordering::Relaxed),
            ),
            line(
                "lkjmc_support_bundles_total",
                "outcome=\"succeeded\"",
                self.inner.bundles_ok.load(Ordering::Relaxed),
            ),
            line(
                "lkjmc_support_bundles_total",
                "outcome=\"failed\"",
                self.inner.bundles_error.load(Ordering::Relaxed),
            ),
        ];
        for (index, bound) in BUCKETS_MS.iter().enumerate() {
            lines.push(line(
                "lkjmc_request_latency_milliseconds_bucket",
                &format!("le=\"{bound}\""),
                self.inner.latency[index].load(Ordering::Relaxed),
            ));
        }
        lines.push(line(
            "lkjmc_request_latency_milliseconds_bucket",
            "le=\"+Inf\"",
            self.inner.latency[6].load(Ordering::Relaxed),
        ));
        debug_assert!(lines.len() <= SERIES_CAP);
        format!("{}\n", lines.join("\n"))
    }
}

fn line(name: &str, label: &str, value: u64) -> String {
    format!("{name}{{{label}}} {value}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn labels_and_series_are_bounded() {
        let text = Metrics::default().export(2);
        assert!(text.lines().count() <= SERIES_CAP);
        for forbidden in ["player", "instance", "requestId", "token", "session"] {
            assert!(!text.contains(forbidden));
        }
    }
}
