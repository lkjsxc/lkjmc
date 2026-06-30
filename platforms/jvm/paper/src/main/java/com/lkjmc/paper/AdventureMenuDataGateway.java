package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.menu.AdventureMenuEntry;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;

final class AdventureMenuDataGateway {
    private final Optional<DaemonClient> daemon;

    AdventureMenuDataGateway(Optional<DaemonClient> daemon) {
        this.daemon = daemon == null ? Optional.empty() : daemon;
    }

    CompletableFuture<List<AdventureMenuEntry>> catalog(Player player) {
        if (daemon.isEmpty()) {
            return CompletableFuture.failedFuture(MenuDataException.missingDaemon());
        }
        var request = new com.lkjmc.common.daemon.DaemonRequest(UUID.randomUUID(),
            new DaemonActor("paper-plugin", player.getName()), "adventure.catalog.list", Map.of());
        return daemon.get().send(request).thenApply(response -> {
            if (!response.ok()) {
                throw MenuDataException.response("adventure.catalog.list", response);
            }
            var entries = new ArrayList<AdventureMenuEntry>();
            for (var value : response.body().getAsJsonArray("adventures")) {
                if (value.isJsonObject()) {
                    entries.add(entry(value.getAsJsonObject()));
                }
            }
            return List.copyOf(entries);
        });
    }

    private static AdventureMenuEntry entry(JsonObject object) {
        return new AdventureMenuEntry(
            text(object, "id", "unknown"),
            text(object, "titleKey", "unknown"),
            text(object, "iconMaterial", "MAP"),
            object.has("pricePoints") ? object.get("pricePoints").getAsLong() : 0,
            object.has("maxPartySize") ? object.get("maxPartySize").getAsInt() : 1,
            !object.has("enabled") || object.get("enabled").getAsBoolean());
    }

    private static String text(JsonObject object, String key, String fallback) {
        return object.has(key) && !object.get(key).isJsonNull() ? object.get(key).getAsString() : fallback;
    }
}
