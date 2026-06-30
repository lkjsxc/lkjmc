package com.lkjmc.common.config;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Map;
import java.util.Optional;
import org.junit.jupiter.api.Test;

final class RuntimeConfigValidatorTest {
    @Test
    void acceptsManagedTokenFileConfig() {
        var result = RuntimeConfigValidator.validate(Map.of(
            "LKJMC_DAEMON_HTTP_URL", "http://127.0.0.1:8765",
            "LKJMC_DAEMON_HTTP_TOKEN_FILE", "/token",
            "LKJMC_INSTANCE_ID", "hub",
            "LKJMC_DEFAULT_LOCALE", "en"
        ), path -> Optional.of("secret"));
        assertTrue(result.valid());
    }

    @Test
    void rejectsInvalidUrlAndInstanceId() {
        assertEquals("schema.invalid_url", RuntimeConfigValidator.validate(Map.of(
            "LKJMC_DAEMON_HTTP_URL", "not a url",
            "LKJMC_DAEMON_HTTP_TOKEN", "secret"
        ), path -> Optional.empty()).code());
        assertEquals("schema.invalid_instance_id", RuntimeConfigValidator.validate(Map.of(
            "LKJMC_INSTANCE_ID", "Bad Id"
        ), path -> Optional.empty()).code());
    }

    @Test
    void rejectsUnreadableTokenFileWhenUrlConfigured() {
        var result = RuntimeConfigValidator.validate(Map.of(
            "LKJMC_DAEMON_HTTP_URL", "http://127.0.0.1:8765",
            "LKJMC_DAEMON_HTTP_TOKEN_FILE", "/missing"
        ), path -> Optional.empty());
        assertEquals("daemon.token_unreadable", result.code());
    }
}
