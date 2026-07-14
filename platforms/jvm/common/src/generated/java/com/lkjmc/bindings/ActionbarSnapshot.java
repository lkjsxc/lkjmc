package com.lkjmc.bindings;

import java.time.Instant;

public record ActionbarSnapshot(
        Instant expiresAt,
        String message,
        long revision
) {
    public ActionbarSnapshot {
        java.util.Objects.requireNonNull(expiresAt, "expiresAt");
        java.util.Objects.requireNonNull(message, "message");
        if (message.isBlank()) throw new IllegalArgumentException("message");
        if (revision <= 0) throw new IllegalArgumentException("revision");
    }
}
