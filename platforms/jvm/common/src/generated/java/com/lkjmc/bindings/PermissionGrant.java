package com.lkjmc.bindings;

import java.time.Instant;
import java.util.UUID;

public record PermissionGrant(
        UUID id,
        String roleId,
        Instant expiresAt
) {
    public PermissionGrant {
        java.util.Objects.requireNonNull(id, "id");
        java.util.Objects.requireNonNull(roleId, "roleId");
        if (roleId.isBlank()) throw new IllegalArgumentException("roleId");
    }
}
