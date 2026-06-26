package com.lkjmc.paper;

import com.lkjmc.common.claim.ClaimChunk;
import com.lkjmc.common.claim.ClaimSnapshot;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.daemon.DaemonResponse;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;

public final class ClaimLiveSmoke {
    private static final String OWNER = "00000000-0000-0000-0000-000000000201";
    private static final String TRUSTED = "00000000-0000-0000-0000-000000000202";
    private final LkjmcPaperPlugin plugin;

    public ClaimLiveSmoke(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
    }

    public void start() {
        if (!"1".equals(System.getenv("LKJMC_PAPER_CLAIM_SMOKE"))) {
            return;
        }
        var daemon = plugin.daemon();
        if (daemon.isEmpty()) {
            plugin.getLogger().severe("lkjmc claim smoke failed: daemon unavailable");
            return;
        }
        run(daemon.get()).thenAccept(message -> plugin.getLogger().info(message))
            .exceptionally(error -> {
                plugin.getLogger().severe("lkjmc claim smoke failed: " + error.getMessage());
                return null;
            });
    }

    private CompletableFuture<String> run(DaemonClient client) {
        return client.send(request("claim.create", createBody()))
            .thenCompose(this::requireOk)
            .thenCompose(created -> {
                var claimId = DaemonJson.string(created.body(), "claimId").orElse("");
                return client.send(request("claim.trust", trustBody()))
                    .thenCompose(this::requireOk)
                    .thenCompose(ignored -> client.send(request("claim.snapshot", Map.of("instanceId", instanceId()))))
                    .thenCompose(this::requireOk)
                    .thenCompose(snapshot -> verifySnapshot(snapshot, claimId))
                    .thenCompose(ignored -> client.send(request("claim.delete", Map.of("claimId", claimId, "operator", true))))
                    .thenCompose(this::requireOk)
                    .thenApply(ignored -> "lkjmc claim smoke passed");
            });
    }

    private CompletableFuture<DaemonResponse> requireOk(DaemonResponse response) {
        if (response.ok()) {
            return CompletableFuture.completedFuture(response);
        }
        var message = response.error().map(error -> error.code() + ": " + error.message()).orElse("daemon error");
        return CompletableFuture.failedFuture(new IllegalStateException(message));
    }

    private CompletableFuture<DaemonResponse> verifySnapshot(DaemonResponse response, String claimId) {
        var snapshot = ClaimSnapshot.fromDaemonBody(response.body());
        var chunk = new ClaimChunk(instanceId(), "world", 7, 9);
        var record = snapshot.at(chunk).orElseThrow(() -> new IllegalStateException("claim missing from snapshot"));
        if (!record.claimId().equals(claimId)
            || !snapshot.decide(OWNER, false, chunk).allowed()
            || !snapshot.decide(TRUSTED, false, chunk).allowed()
            || snapshot.decide("00000000-0000-0000-0000-000000000203", false, chunk).allowed()) {
            return CompletableFuture.failedFuture(new IllegalStateException("claim snapshot decision mismatch"));
        }
        return CompletableFuture.completedFuture(response);
    }

    private DaemonRequest request(String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body);
    }

    private Map<String, Object> createBody() {
        return Map.of(
            "ownerUuid", OWNER, "ownerName", "SmokeOwner", "name", "SmokeBase",
            "instanceId", instanceId(), "worldName", "world", "chunkX", 7, "chunkZ", 9
        );
    }

    private Map<String, Object> trustBody() {
        return Map.of(
            "ownerUuid", OWNER, "trustedUuid", TRUSTED, "trustedName", "SmokeFriend",
            "instanceId", instanceId(), "worldName", "world", "chunkX", 7, "chunkZ", 9
        );
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
