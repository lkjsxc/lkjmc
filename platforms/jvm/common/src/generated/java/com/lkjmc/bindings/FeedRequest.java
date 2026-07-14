package com.lkjmc.bindings;

public record FeedRequest(
        long cursor,
        int limit
) implements SyncRequest {
    public FeedRequest {
        if (cursor < 0) throw new IllegalArgumentException("cursor");
    }
}
