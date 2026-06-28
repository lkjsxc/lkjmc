package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.menu.AchievementMenuEntry;
import com.lkjmc.common.menu.ProfileSummary;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;

final class ProfileMenuDataGateway {
    private final Optional<DaemonClient> daemon;

    ProfileMenuDataGateway(Optional<DaemonClient> daemon) {
        this.daemon = daemon == null ? Optional.empty() : daemon;
    }

    CompletableFuture<ProfileSummary> profile(Player player) {
        return balance(player).thenCombine(achievements(player),
            (balance, achievements) -> new ProfileSummary(balance, achievements.size(), true));
    }

    CompletableFuture<List<AchievementMenuEntry>> achievements(Player player) {
        return request(player, "player.achievements.list", Map.of("playerUuid", player.getUniqueId().toString()))
            .thenApply(body -> {
                var entries = new ArrayList<AchievementMenuEntry>();
                if (!body.has("achievements") || !body.get("achievements").isJsonArray()) {
                    throw MenuDataException.schema("player.achievements.list", "achievements");
                }
                for (var value : body.getAsJsonArray("achievements")) {
                    if (value.isJsonObject()) {
                        var object = value.getAsJsonObject();
                        entries.add(new AchievementMenuEntry(text(object, "id", "unknown"),
                            text(object, "titleKey", "unknown")));
                    }
                }
                return List.copyOf(entries);
            });
    }

    private CompletableFuture<Long> balance(Player player) {
        return request(player, "player.points.balance", Map.of(
            "playerUuid", player.getUniqueId().toString(), "name", player.getName()
        )).thenApply(body -> {
            if (!body.has("balance") || !body.get("balance").isJsonPrimitive()) {
                throw MenuDataException.schema("player.points.balance", "balance");
            }
            return body.get("balance").getAsLong();
        });
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
