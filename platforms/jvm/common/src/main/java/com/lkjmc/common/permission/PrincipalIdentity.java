package com.lkjmc.common.permission;

import java.util.Locale;

public record PrincipalIdentity(String kind, String id, String name) {
    public PrincipalIdentity {
        if (kind == null || kind.isBlank() || id == null || id.isBlank()) {
            throw new IllegalArgumentException("principal kind and id are required");
        }
        kind = kind.toLowerCase(Locale.ROOT);
        id = id.trim();
        name = name == null ? "" : name.trim();
    }

    public String cacheKey() {
        return kind + ":" + id;
    }
}
