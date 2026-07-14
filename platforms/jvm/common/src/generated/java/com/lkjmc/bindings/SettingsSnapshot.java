package com.lkjmc.bindings;

import java.time.Instant;
import java.util.UUID;

public record SettingsSnapshot(
        String actionbarMessage,
        Instant expiresAt,
        UUID playerUuid,
        long revision
) {
    public SettingsSnapshot {
        java.util.Objects.requireNonNull(actionbarMessage, "actionbarMessage");
        if (actionbarMessage.isBlank()) throw new IllegalArgumentException("actionbarMessage");
        java.util.Objects.requireNonNull(expiresAt, "expiresAt");
        java.util.Objects.requireNonNull(playerUuid, "playerUuid");
        if (revision <= 0) throw new IllegalArgumentException("revision");
    }
}
