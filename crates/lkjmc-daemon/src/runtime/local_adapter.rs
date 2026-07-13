use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use crate::runtime::local::LocalRuntime;
use crate::runtime::{RuntimeAdapter, RuntimeObservation};

impl RuntimeAdapter for LocalRuntime {
    fn name(&self) -> &'static str {
        "local-process"
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

    fn logs(&mut self, id: &str, log_root: &str, lines: usize) -> Result<Vec<String>, String> {
        crate::runtime::logs::tail(log_root, id, lines)
    }

    fn delete(&mut self, id: &str) -> Result<RuntimeObservation, String> {
        self.stop(id, Duration::from_secs(3))
    }
}
