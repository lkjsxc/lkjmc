package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.entity.Player;

public final class PointsCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public PointsCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public boolean show(Player player) {
        var instanceId = System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId),
            "player.points.balance",
            Map.of("playerUuid", player.getUniqueId().toString(), "name", player.getName())
        )).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(pointsMessage(player, response.body().get("raw"))))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String pointsMessage(Player player, Object raw) {
        var balance = raw == null ? "0" : extract(raw.toString(), "balance").orElse("0");
        return message(player, "points.balance", Map.of("points", balance));
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }

    private static Optional<String> extract(String json, String key) {
        var needle = "\"" + key + "\":";
        var start = json.indexOf(needle);
        if (start < 0) {
            return Optional.empty();
        }
        var valueStart = start + needle.length();
        var end = valueStart;
        while (end < json.length() && Character.isDigit(json.charAt(end))) {
            end++;
        }
        return end == valueStart ? Optional.empty() : Optional.of(json.substring(valueStart, end));
    }
}
