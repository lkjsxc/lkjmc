package com.lkjmc.paper;

import com.lkjmc.common.claim.ClaimCache;
import com.lkjmc.common.claim.ClaimSnapshot;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import java.time.Duration;
import java.util.Map;
import java.util.UUID;

public final class ClaimSnapshotService {
    private final LkjmcPaperPlugin plugin;
    private final ClaimCache cache;

    public ClaimSnapshotService(LkjmcPaperPlugin plugin, ClaimCache cache) {
        this.plugin = plugin;
        this.cache = cache;
    }

    public void start() {
        refresh();
        plugin.scheduler().runAsyncRepeating(this::refresh, Duration.ofSeconds(5), Duration.ofSeconds(30));
    }

    public void refresh() {
        plugin.daemon().ifPresent(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "claim.snapshot",
            Map.of("instanceId", instanceId())
        )).thenAccept(response -> {
            if (response.ok()) {
                cache.replace(ClaimSnapshot.fromDaemonBody(response.body()));
            }
        }));
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
