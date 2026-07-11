use super::{defaults, BedrockEntry, JavaEntry, PluginsConfig};

impl JavaEntry {
    pub fn preferred_host(&self) -> Option<&str> {
        self.preferred_public_host
            .as_deref()
            .or_else(|| self.public_hosts.first().map(String::as_str))
    }

    pub fn display_host(&self) -> &str {
        self.preferred_host().unwrap_or_else(|| {
            if self.bind_host == "0.0.0.0" || self.bind_host == "::" {
                "127.0.0.1"
            } else {
                self.bind_host.as_str()
            }
        })
    }

    pub fn display_socket(&self) -> String {
        format!("{}:{}", self.display_host(), self.port)
    }
}

impl Default for JavaEntry {
    fn default() -> Self {
        defaults::java_entry()
    }
}

impl Default for BedrockEntry {
    fn default() -> Self {
        defaults::bedrock_entry()
    }
}

impl Default for PluginsConfig {
    fn default() -> Self {
        defaults::plugins()
    }
}
