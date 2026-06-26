package com.lkjmc.paper;

import com.lkjmc.common.claim.ClaimCache;
import com.lkjmc.common.claim.ClaimSnapshot;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import java.time.Duration;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;

public final class ClaimSnapshotService {
    private final LkjmcPaperPlugin plugin;
    private final ClaimCache cache;

    public ClaimSnapshotService(LkjmcPaperPlugin plugin, ClaimCache cache) {
        this.plugin = plugin;
        this.cache = cache;
    }

    public void start() {
        refresh();
        plugin.scheduler().runAsyncRepeating(() -> refresh(), Duration.ofSeconds(5), Duration.ofSeconds(30));
    }

    public CompletableFuture<Boolean> refresh() {
        var daemon = plugin.daemon();
        if (daemon.isEmpty()) {
            return CompletableFuture.completedFuture(false);
        }
        return daemon.get().send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "claim.snapshot",
            Map.of("instanceId", instanceId())
        )).thenApply(response -> {
            if (response.ok()) {
                cache.replace(ClaimSnapshot.fromDaemonBody(response.body()));
                return true;
            }
            return false;
        });
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
