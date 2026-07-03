package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonAccess;
import com.lkjmc.common.daemon.DaemonResponse;

final class MenuDataException extends RuntimeException {
    private final String code;

    MenuDataException(String code, String message) {
        super(message);
        this.code = code;
    }

    String code() {
        return code;
    }

    static MenuDataException missingDaemon() {
        return new MenuDataException(DaemonAccess.fromEnv().code(), "daemon HTTP is not configured");
    }

    static MenuDataException response(String command, DaemonResponse response) {
        var source = response.error().map(error -> error.code()).orElse("daemon.command_failed");
        return new MenuDataException(map(source), command + " failed: " + source);
    }

    static MenuDataException schema(String command, String key) {
        return new MenuDataException("menu.schema_mismatch", command + " missing " + key);
    }

    private static String map(String source) {
        return switch (source) {
            case "daemon.auth_failed" -> "daemon.auth_failed";
            case "daemon.http_failed", "daemon.invalid_json" -> "daemon.http_failed";
            case "command.unknown" -> "daemon.command_unknown";
            case "database.not_configured" -> "database.not_configured";
            case "database.error" -> "database.unavailable";
            default -> source.contains("permission") ? "menu.permission_denied" : "daemon.command_failed";
        };
    }
}
