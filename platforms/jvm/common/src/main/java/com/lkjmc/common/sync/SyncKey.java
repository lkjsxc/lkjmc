package com.lkjmc.common.sync;

import java.nio.charset.StandardCharsets;
import java.util.Set;

public record SyncKey(String domain, String key) {
    private static final Set<String> DOMAINS = Set.of(
            "permissions", "claims", "menus", "profiles", "presence", "routing", "settings");

    public static boolean validDomain(String domain) {
        return DOMAINS.contains(domain);
    }

    public SyncKey {
        if (!DOMAINS.contains(domain)) {
            throw new IllegalArgumentException("unknown sync domain");
        }
        int bytes = key == null ? 0 : key.getBytes(StandardCharsets.UTF_8).length;
        if (bytes == 0 || bytes > 256) {
            throw new IllegalArgumentException("invalid sync key");
        }
    }
}
