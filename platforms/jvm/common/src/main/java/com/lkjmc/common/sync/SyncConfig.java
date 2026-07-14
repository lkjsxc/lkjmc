package com.lkjmc.common.sync;

import java.net.InetAddress;
import java.net.URI;
import java.time.Duration;

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
    public SyncConfig {
        if (credential == null || credential.isBlank()
                || requestTimeout.isNegative() || requestTimeout.isZero()
                || pollInterval.isNegative() || pollInterval.isZero()
                || maxSubscriptions < 1 || maxInflight < 1 || maxEntries < 1
                || maxCacheBytes < 1 || maxResponseBytes < 1) {
            throw new IllegalArgumentException("invalid sync configuration");
        }
        requireLoopback(endpoint);
    }

    public static SyncConfig boundedDefaults(URI endpoint, String credential) {
        return new SyncConfig(endpoint, credential, Duration.ofSeconds(3), Duration.ofSeconds(1),
                Duration.ofSeconds(5), 256, 8, 256, 8 * 1024 * 1024L, 2 * 1024 * 1024);
    }

    private static void requireLoopback(URI endpoint) {
        try {
            if (!"http".equals(endpoint.getScheme()) || endpoint.getHost() == null
                    || !InetAddress.getByName(endpoint.getHost()).isLoopbackAddress()) {
                throw new IllegalArgumentException("sync endpoint must be loopback HTTP");
            }
        } catch (Exception exception) {
            throw new IllegalArgumentException("invalid sync endpoint", exception);
        }
    }
}
