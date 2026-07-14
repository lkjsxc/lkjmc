package com.lkjmc.bindings;

import java.time.Instant;

public record ProfileSnapshot(
        String domain,
        String key,
        long revision,
        Instant generatedAt,
        long credentialRevision,
        ProfilePayload payload
) implements TypedSnapshot {
    public ProfileSnapshot {
        java.util.Objects.requireNonNull(domain, "domain");
        if (domain.isBlank()) throw new IllegalArgumentException("domain");
        java.util.Objects.requireNonNull(key, "key");
        if (key.isBlank()) throw new IllegalArgumentException("key");
        if (revision < 0) throw new IllegalArgumentException("revision");
        java.util.Objects.requireNonNull(generatedAt, "generatedAt");
        if (credentialRevision < 0) throw new IllegalArgumentException("credentialRevision");
        java.util.Objects.requireNonNull(payload, "payload");
    }
}
