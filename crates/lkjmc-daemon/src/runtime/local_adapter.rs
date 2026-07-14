use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use crate::runtime::local::LocalRuntime;
use crate::runtime::{RuntimeAdapter, RuntimeCapabilities, RuntimeObservation};

impl RuntimeAdapter for LocalRuntime {
    fn name(&self) -> &'static str {
        "local-process"
    }

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

    fn runtime_start(
        &self,
        id: &str,
        command: &str,
        args: &[String],
        env: &BTreeMap<String, String>,
        log_root: &str,
        work_dir: &Path,
        deadline: Duration,
    ) -> Result<RuntimeObservation, String> {
        LocalRuntime::runtime_start(self, id, command, args, env, log_root, work_dir, deadline)
    }

    fn runtime_stop(&self, id: &str, deadline: Duration) -> Result<RuntimeObservation, String> {
        LocalRuntime::runtime_stop(self, id, deadline)
    }

    fn runtime_status(&self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        LocalRuntime::runtime_status(self, id)
    }

    fn runtime_adopt(
        &self,
        id: &str,
        identity: crate::runtime::ProcessIdentity,
    ) -> Result<RuntimeObservation, String> {
        Ok(self.recover(id, identity))
    }

    fn runtime_logs(&self, id: &str, log_root: &str, lines: usize) -> Result<Vec<String>, String> {
        crate::runtime::logs::tail(log_root, id, lines)
    }

    fn runtime_delete(&self, id: &str, deadline: Duration) -> Result<RuntimeObservation, String> {
        self.runtime_stop(id, deadline)
    }

    fn runtime_shutdown(&self, deadline: Duration) -> Result<(), String> {
        LocalRuntime::runtime_shutdown(self, deadline)
    }
}
