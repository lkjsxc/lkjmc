use super::types::{
    AssetsConfig, BedrockEntry, BedrockMode, DaemonHttpConfig, FloodgatePluginConfig, JavaEntry,
    LkjmcPluginConfig, PluginInstallConfig, PluginInstallTarget, PluginMode, PluginsConfig,
};

pub(crate) fn network_name() -> String {
    "lkjmc-local".to_string()
}

pub(crate) fn forwarding_secret_file() -> String {
    "/etc/lkjmc/forwarding.secret".to_string()
}

pub(crate) fn database_pool_size() -> u32 {
    8
}

pub(crate) fn java_entry() -> JavaEntry {
    JavaEntry {
        bind_host: "127.0.0.1".to_string(),
        port: 25565,
        public_hosts: Vec::new(),
        preferred_public_host: None,
    }
}

pub(crate) fn bedrock_entry() -> BedrockEntry {
    BedrockEntry {
        mode: BedrockMode::Auto,
        host: "127.0.0.1".to_string(),
        port: 19132,
    }
}

pub(crate) fn daemon_http() -> DaemonHttpConfig {
    DaemonHttpConfig {
        enabled: true,
        address: "127.0.0.1:8765".to_string(),
        token_file: "/etc/lkjmc/daemon-http.token".to_string(),
    }
}

pub(crate) fn assets() -> AssetsConfig {
    AssetsConfig {
        root: "/opt/lkjmc/assets".to_string(),
        server_channel: "stable".to_string(),
        plugin_channel: "stable".to_string(),
        user_agent: "lkjmc (+https://github.com/lkjsxc/lkjmc)".to_string(),
        download_timeout_seconds: 120,
    }
}

pub(crate) fn plugins() -> PluginsConfig {
    PluginsConfig {
        lkjmc: lkjmc_plugin(),
        viaversion: backend_plugin(),
        viabackwards: backend_plugin(),
        geyser: proxy_plugin(),
        floodgate: floodgate_plugin(),
    }
}

pub(crate) fn lkjmc_plugin() -> LkjmcPluginConfig {
    LkjmcPluginConfig { enabled: true }
}

pub(crate) fn backend_plugin() -> PluginInstallConfig {
    PluginInstallConfig {
        mode: PluginMode::Auto,
        install_on: PluginInstallTarget::Backend,
    }
}

pub(crate) fn proxy_plugin() -> PluginInstallConfig {
    PluginInstallConfig {
        mode: PluginMode::Auto,
        install_on: PluginInstallTarget::Proxy,
    }
}

pub(crate) fn floodgate_plugin() -> FloodgatePluginConfig {
    FloodgatePluginConfig {
        mode: PluginMode::Auto,
        install_on: PluginInstallTarget::Proxy,
        backend_api: false,
    }
}

pub(crate) fn proxy_java_memory_mb() -> u32 {
    512
}

pub(crate) fn port_range_start() -> u16 {
    25566
}

pub(crate) fn port_range_end() -> u16 {
    25665
}
