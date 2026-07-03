package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.menu.RandomTeleportQuote;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;

final class RandomTeleportMenuGateway {
    private final Optional<DaemonClient> daemon;

    RandomTeleportMenuGateway(Optional<DaemonClient> daemon) {
        this.daemon = daemon == null ? Optional.empty() : daemon;
    }

    CompletableFuture<RandomTeleportQuote> quote(Player player) {
        return quote(player, "overworld");
    }

    CompletableFuture<RandomTeleportQuote> quote(Player player, String profileId) {
        return request(player, profileId).thenApply(body -> new RandomTeleportQuote(
            text(body, "profileId", "overworld"), text(body, "targetEnvironment", "normal"),
            bool(body, "confirmationRequired"), bool(body, "enabled"), bool(body, "canAfford"),
            integer(body, "costPoints"), integer(body, "balance"), integer(body, "cooldownRemainingSeconds"),
            (int) integer(body, "minRadius"), (int) integer(body, "maxRadius"),
            (int) integer(body, "maxAttempts")));
    }

    private CompletableFuture<JsonObject> request(Player player, String profileId) {
        if (daemon.isEmpty()) {
            return CompletableFuture.failedFuture(MenuDataException.missingDaemon());
        }
        var request = new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", player.getName()),
            "player.random-teleport.quote", Map.of("playerUuid", player.getUniqueId().toString(),
                "serverId", instanceId(), "profileId", profileId));
        return daemon.get().send(request).thenApply(response -> {
            if (!response.ok()) {
                throw MenuDataException.response("player.random-teleport.quote", response);
            }
            return response.body();
        });
    }

    private static long integer(JsonObject object, String key) {
        return object.has(key) && !object.get(key).isJsonNull() ? object.get(key).getAsLong() : 0;
    }

    private static boolean bool(JsonObject object, String key) {
        return object.has(key) && !object.get(key).isJsonNull() && object.get(key).getAsBoolean();
    }

    private static String text(JsonObject object, String key, String fallback) {
        return object.has(key) && !object.get(key).isJsonNull() ? object.get(key).getAsString() : fallback;
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
