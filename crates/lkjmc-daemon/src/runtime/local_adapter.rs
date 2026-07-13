use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use crate::runtime::local::LocalRuntime;
use crate::runtime::{RuntimeAdapter, RuntimeCapabilities, RuntimeObservation};

impl RuntimeAdapter for LocalRuntime {
    fn name(&self) -> &'static str { "local-process" }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            process_identity: true,
            readiness: true,
            storage: false,
            secrets: false,
            configuration: true,
            logs: true,
            recovery: true,
        }
    }

    fn check_capabilities(&self) -> Result<(), String> {
        std::fs::metadata("/proc/self/stat")
            .map(|_| ())
            .map_err(|error| format!("local process identity unsupported: {error}"))
    }

    fn start(
        &self,
        id: &str,
        command: &str,
        args: &[String],
        env: &BTreeMap<String, String>,
        log_root: &str,
        work_dir: &Path,
        deadline: Duration,
    ) -> Result<RuntimeObservation, String> {
        LocalRuntime::start(self, id, command, args, env, log_root, work_dir, deadline)
    }

    fn stop(&self, id: &str, deadline: Duration) -> Result<RuntimeObservation, String> {
        LocalRuntime::stop(self, id, deadline)
    }

    fn status(&self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        LocalRuntime::status(self, id)
    }

    fn logs(&self, id: &str, log_root: &str, lines: usize) -> Result<Vec<String>, String> {
        crate::runtime::logs::tail(log_root, id, lines)
    }

    fn delete(&self, id: &str, deadline: Duration) -> Result<RuntimeObservation, String> {
        self.stop(id, deadline)
    }

    fn shutdown(&self, deadline: Duration) -> Result<(), String> {
        LocalRuntime::shutdown(self, deadline)
    }
}
