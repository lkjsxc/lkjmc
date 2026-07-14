package com.lkjmc.bindings;

import java.time.Instant;
import java.util.Set;

public record ClaimSnapshot(
        Set<ClaimChunk> chunks,
        Instant expiresAt,
        long revision
) {
    public ClaimSnapshot {
        chunks = Set.copyOf(chunks);
        java.util.Objects.requireNonNull(expiresAt, "expiresAt");
        if (revision <= 0) throw new IllegalArgumentException("revision");
    }
}
