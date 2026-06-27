package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.menu.ClaimMenuEntry;
import com.lkjmc.common.menu.ServerMenuEntry;
import com.lkjmc.common.menu.ShopMenuEntry;
import com.lkjmc.common.menu.TravelMenuEntry;
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
        return request(player, "instance.list", Map.of()).thenApply(body -> {
            var entries = new ArrayList<ServerMenuEntry>();
            for (var value : body.getAsJsonArray("instances")) {
                if (value.isJsonObject()) {
                    entries.add(server(value.getAsJsonObject()));
                }
            }
            return List.copyOf(entries);
        });
    }

    CompletableFuture<List<TravelMenuEntry>> homes(Player player) {
        return travel(player, "player.home.list", Map.of("playerUuid", player.getUniqueId().toString()), "homes", "home");
    }

    CompletableFuture<List<TravelMenuEntry>> warps(Player player) {
        return travel(player, "player.warp.list", Map.of(), "warps", "warp");
    }

    CompletableFuture<List<ClaimMenuEntry>> claims(Player player) {
        return request(player, "claim.list", Map.of("ownerUuid", player.getUniqueId().toString())).thenApply(body -> {
            var entries = new ArrayList<ClaimMenuEntry>();
            for (var value : body.getAsJsonArray("claims")) {
                if (value.isJsonObject()) {
                    var object = value.getAsJsonObject();
                    entries.add(new ClaimMenuEntry(text(object, "name", "unknown"),
                        object.has("chunkCount") ? object.get("chunkCount").getAsLong() : 0));
                }
            }
            return List.copyOf(entries);
        });
    }

    CompletableFuture<List<ShopMenuEntry>> shop(Player player) {
        return request(player, "player.shop.list", Map.of()).thenApply(body -> {
            var entries = new ArrayList<ShopMenuEntry>();
            for (var value : body.getAsJsonArray("items")) {
                if (value.isJsonObject()) {
                    var object = value.getAsJsonObject();
                    entries.add(new ShopMenuEntry(text(object, "id", "unknown"),
                        text(object, "titleKey", "unknown"),
                        object.has("pricePoints") ? object.get("pricePoints").getAsLong() : 0));
                }
            }
            return List.copyOf(entries);
        });
    }

    private CompletableFuture<List<TravelMenuEntry>> travel(Player player, String command,
                                                            Map<String, Object> body,
                                                            String array, String nameKey) {
        return request(player, command, body).thenApply(response -> {
            var entries = new ArrayList<TravelMenuEntry>();
            for (var value : response.getAsJsonArray(array)) {
                if (value.isJsonObject()) {
                    var object = value.getAsJsonObject();
                    entries.add(new TravelMenuEntry(text(object, nameKey, "unknown"), text(object, "serverId", "unknown")));
                }
            }
            return List.copyOf(entries);
        });
    }

    private CompletableFuture<JsonObject> request(Player player, String command, Map<String, Object> body) {
        if (daemon.isEmpty()) {
            return CompletableFuture.failedFuture(new IllegalStateException("daemon unavailable"));
        }
        var request = new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", player.getName()), command, body);
        return daemon.get().send(request).thenApply(response -> {
            if (!response.ok()) {
                throw new IllegalStateException(command + " failed");
            }
            return response.body();
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
