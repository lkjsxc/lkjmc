use super::AppState;

impl AppState {
    pub fn verify_web_bootstrap(&self, presented: &str) -> bool {
        self.secrets.verify_current(presented)
    }

    pub fn web_bootstrap_configured(&self) -> bool {
        self.secrets.configured()
    }

    pub fn web_bootstrap_fingerprint(&self) -> Option<String> {
        self.secrets.fingerprint()
    }

    pub fn current_web_bootstrap(&self) -> Option<String> {
        self.secrets.current()
    }

    #[cfg(test)]
    pub fn set_web_bootstrap(&self, value: String) -> Result<(), String> {
        self.secrets.replace(Some(value))
    }

    pub fn stage_web_bootstrap(&self, value: String, previous: String) -> Result<(), String> {
        self.secrets.stage(value, previous)
    }

    pub fn retire_previous_web_bootstrap(&self) -> Result<(), String> {
        self.secrets.retire_previous()
    }

    pub fn restore_web_bootstrap(&self, value: String) -> Result<(), String> {
        self.secrets.replace(Some(value))
    }

    pub fn clear_web_bootstrap(&self) -> Result<(), String> {
        self.secrets.replace(None)
    }

    #[cfg(test)]
    pub fn previous_web_bootstrap(&self) -> Option<String> {
        self.secrets.previous()
    }

    pub fn authenticate_credential(
        &self,
        credential: &str,
    ) -> Result<Option<crate::authz::AuthenticatedSubject>, lkjmc_store::error::StoreError> {
        if credential.trim().is_empty() {
            return Ok(None);
        }
        let mut client = self.request_database_connection()?;
        let record = self
            .credential_cache
            .authenticate(&mut client, credential)?;
        Ok(record.map(crate::authz::AuthenticatedSubject::credential))
    }
}
