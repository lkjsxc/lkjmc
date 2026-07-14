package com.lkjmc.common.sync;

import java.net.URI;
import java.time.Duration;
import java.util.Locale;
import java.util.Set;

public record SyncConfig(
        URI endpoint,
        String credential,
        Duration requestTimeout,
        Duration pollInterval,
        Duration maxAge,
        int maxSubscriptions,
        int maxInflight,
        int maxEntries,
        long maxCacheBytes,
        int maxResponseBytes) {
    private static final Set<String> LOOPBACK_HOSTS = Set.of("localhost", "127.0.0.1", "::1", "[::1]");

    public SyncConfig {
        if (endpoint == null || credential == null || credential.isBlank()
                || requestTimeout == null || requestTimeout.isNegative() || requestTimeout.isZero()
                || pollInterval == null || pollInterval.isNegative() || pollInterval.isZero()
                || maxAge == null || maxAge.isNegative() || maxAge.isZero()
                || maxSubscriptions < 1 || maxInflight < 1 || maxEntries < 1
                || maxEntries > maxSubscriptions || maxCacheBytes < 1 || maxResponseBytes < 1) {
            throw new IllegalArgumentException("invalid sync configuration");
        }
        requireLoopback(endpoint);
    }

    public static SyncConfig boundedDefaults(URI endpoint, String credential) {
        return new SyncConfig(endpoint, credential, Duration.ofSeconds(3), Duration.ofSeconds(1),
                Duration.ofSeconds(5), 256, 8, 256, 8 * 1024 * 1024L, 2 * 1024 * 1024);
    }

    private static void requireLoopback(URI endpoint) {
        String host = endpoint.getHost();
        boolean safe = "http".equals(endpoint.getScheme()) && host != null
                && LOOPBACK_HOSTS.contains(host.toLowerCase(Locale.ROOT))
                && endpoint.getUserInfo() == null && endpoint.getQuery() == null
                && endpoint.getFragment() == null;
        if (!safe) {
            throw new IllegalArgumentException("sync endpoint must be literal loopback HTTP");
        }
    }
}
