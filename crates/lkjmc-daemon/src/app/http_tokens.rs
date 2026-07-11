use super::AppState;

impl AppState {
    #[rustfmt::skip]
    pub fn http_token(&self) -> Option<String> { self.option(|c| c.http_token.clone()) }
    #[rustfmt::skip]
    pub fn http_previous_token(&self) -> Option<String> { self.option(|c| c.http_previous_token.clone()) }

    #[cfg(test)]
    pub fn set_http_token(&self, value: String) -> Result<(), String> {
        let mut config = self
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        config.http_token = Some(value);
        config.http_previous_token = None;
        Ok(())
    }

    pub fn stage_http_token(&self, value: String, previous: String) -> Result<(), String> {
        let mut config = self
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        config.http_token = Some(value);
        config.http_previous_token = Some(previous);
        Ok(())
    }

    pub fn retire_previous_http_token(&self) -> Result<(), String> {
        let mut config = self
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        config.http_previous_token = None;
        Ok(())
    }

    pub fn restore_http_token(&self, value: String) -> Result<(), String> {
        self.set_tokens(Some(value))
    }

    pub fn clear_http_tokens(&self) -> Result<(), String> {
        self.set_tokens(None)
    }

    fn set_tokens(&self, value: Option<String>) -> Result<(), String> {
        let mut config = self
            .config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?;
        config.http_token = value;
        config.http_previous_token = None;
        Ok(())
    }
}
