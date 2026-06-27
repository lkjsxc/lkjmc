package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.menu.ServerMenuEntry;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;

final class MenuDataGateway {
    private final Optional<DaemonClient> daemon;

    MenuDataGateway(Optional<DaemonClient> daemon) {
        this.daemon = daemon == null ? Optional.empty() : daemon;
    }

    CompletableFuture<List<ServerMenuEntry>> servers(Player player) {
        if (daemon.isEmpty()) {
            return CompletableFuture.failedFuture(new IllegalStateException("daemon unavailable"));
        }
        var request = new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", player.getName()),
            "instance.list", Map.of());
        return daemon.get().send(request).thenApply(response -> {
            if (!response.ok()) {
                throw new IllegalStateException("instance.list failed");
            }
            var entries = new ArrayList<ServerMenuEntry>();
            for (var value : response.body().getAsJsonArray("instances")) {
                if (value.isJsonObject()) {
                    entries.add(server(value.getAsJsonObject()));
                }
            }
            return List.copyOf(entries);
        });
    }

    private static ServerMenuEntry server(JsonObject object) {
        var presence = object.has("presence") && object.get("presence").isJsonObject()
            ? object.getAsJsonObject("presence") : null;
        return new ServerMenuEntry(
            text(object, "id", "unknown"),
            text(object, "kind", "unknown"),
            text(object, "desiredState", "unknown"),
            text(object, "observedState", "unknown"),
            object.has("healthy") && !object.get("healthy").isJsonNull() && object.get("healthy").getAsBoolean(),
            presence == null || !presence.has("playerCount") || presence.get("playerCount").isJsonNull()
                ? null : presence.get("playerCount").getAsInt()
        );
    }

    private static String text(JsonObject object, String key, String fallback) {
        return object.has(key) && !object.get(key).isJsonNull() ? object.get(key).getAsString() : fallback;
    }
}
