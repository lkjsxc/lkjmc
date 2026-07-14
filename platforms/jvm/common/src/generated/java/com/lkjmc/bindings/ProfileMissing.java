package com.lkjmc.bindings;

import java.util.UUID;

public record ProfileMissing(
        UUID playerUuid,
        String scope
) implements ProfilePayload {
    public ProfileMissing {
        java.util.Objects.requireNonNull(playerUuid, "playerUuid");
        java.util.Objects.requireNonNull(scope, "scope");
        if (scope.isBlank()) throw new IllegalArgumentException("scope");
    }
}
