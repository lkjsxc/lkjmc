use super::AppState;

impl AppState {
    pub fn set_unix_peer_policy(
        &self,
        policy: crate::transport::peer::UnixPeerPolicy,
    ) -> Result<(), String> {
        let mut config = self
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        config.unix_peer_policy = Some(policy);
        Ok(())
    }

    pub fn unix_peer_policy(&self) -> Option<crate::transport::peer::UnixPeerPolicy> {
        self.config
            .read()
            .ok()
            .and_then(|config| config.unix_peer_policy.clone())
    }
}
