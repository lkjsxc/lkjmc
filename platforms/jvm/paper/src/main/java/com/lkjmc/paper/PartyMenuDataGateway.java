package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.menu.PartyStatus;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;

final class PartyMenuDataGateway {
    private final Optional<DaemonClient> daemon;

    PartyMenuDataGateway(Optional<DaemonClient> daemon) {
        this.daemon = daemon == null ? Optional.empty() : daemon;
    }

    CompletableFuture<PartyStatus> party(Player player) {
        return request(player, "player.party.info", Map.of("playerUuid", player.getUniqueId().toString()))
            .thenApply(body -> new PartyStatus(
                body.has("found") && body.get("found").getAsBoolean(),
                text(body, "name", "party"), text(body, "role", "member"), true));
    }

    private CompletableFuture<JsonObject> request(Player player, String command, Map<String, Object> body) {
        if (daemon.isEmpty()) {
            return CompletableFuture.failedFuture(MenuDataException.missingDaemon());
        }
        var actor = new DaemonActor("paper-plugin", player.getName());
        var request = new DaemonRequest(UUID.randomUUID(), actor, command, body);
        return daemon.get().send(request).thenApply(response -> {
            if (!response.ok()) {
                throw MenuDataException.response(command, response);
            }
            return response.body();
        });
    }

    private static String text(JsonObject object, String key, String fallback) {
        return object.has(key) && !object.get(key).isJsonNull() ? object.get(key).getAsString() : fallback;
    }
}
