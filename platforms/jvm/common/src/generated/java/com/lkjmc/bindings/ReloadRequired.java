package com.lkjmc.bindings;

public record ReloadRequired(long cursor, long activeFloor, long credentialRevision) implements SyncResponse {}
