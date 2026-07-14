package com.lkjmc.bindings;

import java.util.List;

public record FeedResponse(
        long cursor,
        long activeFloor,
        long credentialRevision,
        List<FeedChange> changes
) implements SyncResponse {
    public FeedResponse {
        if (cursor < 0) throw new IllegalArgumentException("cursor");
        if (activeFloor < 0) throw new IllegalArgumentException("activeFloor");
        if (credentialRevision < 0) throw new IllegalArgumentException("credentialRevision");
        changes = List.copyOf(changes);
    }
}
