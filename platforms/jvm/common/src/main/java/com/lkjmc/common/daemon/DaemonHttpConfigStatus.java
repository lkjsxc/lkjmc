package com.lkjmc.common.daemon;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;

public record DaemonHttpConfigStatus(String code, boolean configured) {
    public static final String CONFIGURED = "daemon.configured";
    public static final String NOT_CONFIGURED = "daemon.not_configured";
    public static final String TOKEN_MISSING = "daemon.token_missing";
    public static final String TOKEN_UNREADABLE = "daemon.token_unreadable";

    public static DaemonHttpConfigStatus fromEnv() {
        return from(System.getenv(), DaemonHttpConfigStatus::readTokenFileSafe);
    }

    public static DaemonHttpConfigStatus from(Map<String, String> env, Function<String, Optional<String>> reader) {
        var url = value(env, "LKJMC_DAEMON_HTTP_URL");
        if (url.isEmpty()) {
            return new DaemonHttpConfigStatus(NOT_CONFIGURED, false);
        }
        if (value(env, "LKJMC_DAEMON_HTTP_TOKEN").isPresent()) {
            return new DaemonHttpConfigStatus(CONFIGURED, true);
        }
        var tokenFile = value(env, "LKJMC_DAEMON_HTTP_TOKEN_FILE");
        if (tokenFile.isEmpty()) {
            return new DaemonHttpConfigStatus(TOKEN_MISSING, false);
        }
        var token = reader.apply(tokenFile.get()).map(String::trim).filter(v -> !v.isBlank());
        return token.isPresent()
            ? new DaemonHttpConfigStatus(CONFIGURED, true)
            : new DaemonHttpConfigStatus(TOKEN_UNREADABLE, false);
    }

    private static Optional<String> value(Map<String, String> env, String key) {
        return Optional.ofNullable(env.get(key)).map(String::trim).filter(value -> !value.isBlank());
    }

    public static Optional<String> readTokenFileSafe(String path) {
        try {
            return Optional.of(Files.readString(Path.of(path), StandardCharsets.UTF_8));
        } catch (IOException error) {
            return Optional.empty();
        }
    }
}
