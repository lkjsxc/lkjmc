package com.lkjmc.bindings;

import java.util.List;
public record FeedResponse(long cursor, long activeFloor, long credentialRevision,
                           List<FeedChange> changes) implements SyncResponse {
    public FeedResponse { changes = List.copyOf(changes); }
}
