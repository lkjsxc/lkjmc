package com.lkjmc.bindings;

import java.util.Objects;

public record SnapshotRequest(String domain, String key) implements SyncRequest {
    public SnapshotRequest {
        Objects.requireNonNull(domain, "domain");
        Objects.requireNonNull(key, "key");
    }
}
