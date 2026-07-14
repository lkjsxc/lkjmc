package com.lkjmc.bindings;

public record SnapshotRequest(
        String domain,
        String key
) implements SyncRequest {
    public SnapshotRequest {
        java.util.Objects.requireNonNull(domain, "domain");
        if (domain.isBlank()) throw new IllegalArgumentException("domain");
        java.util.Objects.requireNonNull(key, "key");
        if (key.isBlank()) throw new IllegalArgumentException("key");
    }
}
