package com.lkjmc.common.sync;

import com.lkjmc.bindings.TypedSnapshot;
import java.time.Instant;

public record SyncSnapshot(
        TypedSnapshot value,
        int encodedBytes,
        Instant receivedAt) {
    public SyncSnapshot {
        if (value == null || value.revision() <= 0 || encodedBytes <= 0 || receivedAt == null) {
            throw new IllegalArgumentException("invalid snapshot bounds");
        }
    }

    public SyncKey key() {
        return new SyncKey(value.domain(), value.key());
    }

    public long revision() {
        return value.revision();
    }
}
