package com.lkjmc.common.sync;

import java.net.URI;
import java.util.Map;
import java.util.Optional;

public final class SyncBootstrap {
    private SyncBootstrap() {}

    public static Optional<SyncCoordinator> fromEnvironment(Map<String, String> environment) {
        String endpoint = environment.get("LKJMC_SYNC_ENDPOINT");
        String credential = environment.get("LKJMC_SYNC_CREDENTIAL");
        if (endpoint == null && credential == null) {
            return Optional.empty();
        }
        if (endpoint == null || credential == null) {
            throw new IllegalStateException("sync endpoint and credential must be configured together");
        }
        return Optional.of(new SyncCoordinator(SyncConfig.boundedDefaults(URI.create(endpoint), credential)));
    }
}
