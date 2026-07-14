package com.lkjmc.bindings;

public record SnapshotUnavailable(
        String domain,
        String key,
        long credentialRevision,
        String reason
) implements SyncResponse {
    public SnapshotUnavailable {
        java.util.Objects.requireNonNull(domain, "domain");
        if (domain.isBlank()) throw new IllegalArgumentException("domain");
        java.util.Objects.requireNonNull(key, "key");
        if (key.isBlank()) throw new IllegalArgumentException("key");
        if (credentialRevision < 0) throw new IllegalArgumentException("credentialRevision");
        java.util.Objects.requireNonNull(reason, "reason");
        if (reason.isBlank()) throw new IllegalArgumentException("reason");
    }
}
