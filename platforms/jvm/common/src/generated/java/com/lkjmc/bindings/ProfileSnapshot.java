package com.lkjmc.bindings;

import java.util.List;
import java.util.UUID;

public record ProfileSnapshot(
        long fence,
        List<ItemSlot> inventory,
        UUID playerUuid,
        long profileRevision,
        String scope
) {
    public ProfileSnapshot {
        if (fence <= 0) throw new IllegalArgumentException("fence");
        inventory = List.copyOf(inventory);
        java.util.Objects.requireNonNull(playerUuid, "playerUuid");
        if (profileRevision <= 0) throw new IllegalArgumentException("profileRevision");
        java.util.Objects.requireNonNull(scope, "scope");
        if (scope.isBlank()) throw new IllegalArgumentException("scope");
    }
}
