package com.lkjmc.bindings;

public record FeedChange(
        long feedRevision,
        String domain,
        String key,
        long revision
) {
    public FeedChange {
        if (feedRevision < 0) throw new IllegalArgumentException("feedRevision");
        java.util.Objects.requireNonNull(domain, "domain");
        if (domain.isBlank()) throw new IllegalArgumentException("domain");
        java.util.Objects.requireNonNull(key, "key");
        if (key.isBlank()) throw new IllegalArgumentException("key");
        if (revision < 0) throw new IllegalArgumentException("revision");
    }
}
