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

    public boolean show(Player player, String[] args) {
        if (args.length == 1 && args[0].equalsIgnoreCase("top")) {
            return call(player, "player.points.top", Map.of("limit", 10), true);
        }
        return call(player, "player.points.balance", Map.of(
            "playerUuid", player.getUniqueId().toString(), "name", player.getName()
        ), false);
    }

    private boolean call(Player player, String command, Map<String, Object> body, boolean top) {
        var instanceId = System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId), command, body
        )).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(top ? topMessage(player, response.body().get("raw"))
                : pointsMessage(player, response.body().get("raw"))))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String pointsMessage(Player player, Object raw) {
        var balance = raw == null ? "0" : extractNumber(raw.toString(), "balance").orElse("0");
        return message(player, "points.balance", Map.of("points", balance));
    }

    private String topMessage(Player player, Object raw) {
        var text = raw == null ? "" : raw.toString();
        var name = extractString(text, "name").orElse("-");
        var balance = extractNumber(text, "balance").orElse("0");
        return message(player, "points.top", Map.of("name", name, "points", balance));
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }

    private static Optional<String> extractNumber(String json, String key) {
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

    private static Optional<String> extractString(String json, String key) {
        var needle = "\"" + key + "\":\"";
        var start = json.indexOf(needle);
        if (start < 0) {
            return Optional.empty();
        }
        var valueStart = start + needle.length();
        var end = json.indexOf('"', valueStart);
        return end < 0 ? Optional.empty() : Optional.of(json.substring(valueStart, end));
    }
}
