package com.lkjmc.bindings;

import java.time.Instant;
public record TypedSnapshot(String domain, String key, long revision, Instant generatedAt,
                            long credentialRevision, Object payload) implements SyncResponse {}
