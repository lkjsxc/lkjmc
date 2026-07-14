package com.lkjmc.bindings;

import java.util.List;

public record PermissionPayload(
        String principalKind,
        String principalId,
        List<PermissionGrant> grants,
        List<String> permissions
) implements DomainPayload {
    public PermissionPayload {
        java.util.Objects.requireNonNull(principalKind, "principalKind");
        if (principalKind.isBlank()) throw new IllegalArgumentException("principalKind");
        java.util.Objects.requireNonNull(principalId, "principalId");
        if (principalId.isBlank()) throw new IllegalArgumentException("principalId");
        grants = List.copyOf(grants);
        permissions = List.copyOf(permissions);
    }
}
