use std::sync::OnceLock;

use lkjmc_core::id::CommandId;

use super::RequestAdmission;

pub(super) struct Correlation(OnceLock<CommandId>);

impl Correlation {
    pub(super) fn new() -> Self {
        Self(OnceLock::new())
    }
}

impl RequestAdmission {
    pub(crate) fn correlate(&self, request_id: CommandId) {
        let _ = self.lease.correlation.0.set(request_id);
    }

    pub(crate) fn request_id(&self) -> Option<CommandId> {
        self.lease.correlation.0.get().cloned()
    }
}
