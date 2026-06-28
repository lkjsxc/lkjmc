package com.lkjmc.common.menu;

import java.util.Map;

public record MenuDiagnostic(String code, String nameKey, String loreKey) {
    private static final Map<String, String> SUFFIXES = Map.ofEntries(
        Map.entry("daemon.not_configured", "daemon.not-configured"),
        Map.entry("daemon.token_missing", "daemon.token-missing"),
        Map.entry("daemon.token_unreadable", "daemon.token-unreadable"),
        Map.entry("daemon.http_failed", "daemon.http-failed"),
        Map.entry("daemon.auth_failed", "daemon.auth-failed"),
        Map.entry("daemon.command_unknown", "daemon.command-unknown"),
        Map.entry("daemon.command_failed", "daemon.command-failed"),
        Map.entry("database.not_configured", "database.not-configured"),
        Map.entry("database.unavailable", "database.unavailable"),
        Map.entry("menu.schema_mismatch", "menu.schema-mismatch"),
        Map.entry("menu.permission_denied", "menu.permission-denied")
    );

    public static MenuDiagnostic of(String code) {
        var safeCode = SUFFIXES.containsKey(code) ? code : "daemon.command_failed";
        var key = "menu.unavailable." + SUFFIXES.get(safeCode);
        return new MenuDiagnostic(safeCode, key, key + ".lore");
    }
}
