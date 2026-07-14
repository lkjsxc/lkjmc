package com.lkjmc.bindings;

import java.util.Objects;

public record FeedRequest(long cursor, int limit) implements SyncRequest {
    public FeedRequest {
    }
}
