package com.lkjmc.common.permission;

import java.time.Instant;
import java.util.Set;

public record PermissionSnapshot(
    PrincipalIdentity principal,
    Set<String> permissions,
    Instant fetchedAt,
    Instant validUntil
) {
    public PermissionSnapshot {
        if (principal == null || fetchedAt == null || validUntil == null) {
            throw new IllegalArgumentException("principal and timestamps are required");
        }
        permissions = Set.copyOf(permissions == null ? Set.of() : permissions);
    }

    public boolean fresh(Instant now) {
        return !validUntil.isBefore(now == null ? Instant.now() : now);
    }

    public boolean grants(String permission) {
        return permissions.contains(permission) || permissions.contains(PermissionNodes.ADMIN_ADMIN);
    }
}
