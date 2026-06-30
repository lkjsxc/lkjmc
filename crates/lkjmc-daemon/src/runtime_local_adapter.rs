use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use crate::runtime::{RuntimeAdapter, RuntimeCapabilities, RuntimeObservation};
use crate::runtime_local::LocalRuntime;

impl RuntimeAdapter for LocalRuntime {
    fn name(&self) -> &'static str {
        "local-process"
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities::local_process()
    }

    fn recover(&mut self, id: &str, pid: u32) -> RuntimeObservation {
        LocalRuntime::recover(self, id, pid)
    }

    fn start(
        &mut self,
        id: &str,
        command: &str,
        args: &[String],
        env: &BTreeMap<String, String>,
        log_root: &str,
        work_dir: &Path,
    ) -> Result<RuntimeObservation, String> {
        LocalRuntime::start(self, id, command, args, env, log_root, work_dir)
    }

    fn stop(&mut self, id: &str, timeout: Duration) -> Result<RuntimeObservation, String> {
        LocalRuntime::stop(self, id, timeout)
    }

    fn status(&mut self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        LocalRuntime::status(self, id)
    }
}
