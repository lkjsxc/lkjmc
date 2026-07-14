package com.lkjmc.common.sync;

import com.google.gson.JsonElement;
import java.time.Instant;

public record SyncSnapshot(
        SyncKey key,
        long revision,
        Instant generatedAt,
        JsonElement payload,
        int encodedBytes,
        Instant receivedAt) {
    public SyncSnapshot {
        if (revision <= 0 || encodedBytes <= 0) {
            throw new IllegalArgumentException("invalid snapshot bounds");
        }
        payload = payload.deepCopy();
    }

    @Override
    public JsonElement payload() {
        return payload.deepCopy();
    }
}
