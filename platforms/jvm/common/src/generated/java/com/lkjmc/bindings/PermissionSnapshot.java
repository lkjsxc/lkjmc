package com.lkjmc.bindings;

import java.time.Instant;
import java.util.Set;

public record PermissionSnapshot(
        Instant expiresAt,
        Set<String> permissions,
        String principal,
        long revision
) {
    public PermissionSnapshot {
        java.util.Objects.requireNonNull(expiresAt, "expiresAt");
        permissions = Set.copyOf(permissions);
        java.util.Objects.requireNonNull(principal, "principal");
        if (principal.isBlank()) throw new IllegalArgumentException("principal");
        if (revision <= 0) throw new IllegalArgumentException("revision");
    }
}
