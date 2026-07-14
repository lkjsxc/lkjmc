package com.lkjmc.bindings;

public record ReloadRequired(
        long cursor,
        long activeFloor,
        long credentialRevision
) implements SyncResponse {
    public ReloadRequired {
        if (cursor < 0) throw new IllegalArgumentException("cursor");
        if (activeFloor < 0) throw new IllegalArgumentException("activeFloor");
        if (credentialRevision < 0) throw new IllegalArgumentException("credentialRevision");
    }
}
