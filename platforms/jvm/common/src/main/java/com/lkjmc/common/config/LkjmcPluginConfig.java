package com.lkjmc.common.config;

public record LkjmcPluginConfig(String daemonUrl, String locale) {
    public LkjmcPluginConfig {
        if (locale == null || locale.isBlank()) {
            locale = "en";
        }
    }
}
