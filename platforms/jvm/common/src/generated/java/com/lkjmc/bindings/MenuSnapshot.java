package com.lkjmc.bindings;

import java.time.Instant;
import java.util.List;

public record MenuSnapshot(
        List<String> entries,
        Instant expiresAt,
        long revision
) {
    public MenuSnapshot {
        entries = List.copyOf(entries);
        java.util.Objects.requireNonNull(expiresAt, "expiresAt");
        if (revision <= 0) throw new IllegalArgumentException("revision");
    }
}
