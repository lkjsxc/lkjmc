package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
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
            () -> player.sendMessage(top ? topMessage(player, response.body()) : pointsMessage(player, response.body())))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String pointsMessage(Player player, JsonObject body) {
        var balance = DaemonJson.integer(body, "balance").map(String::valueOf).orElse("0");
        return message(player, "points.balance", Map.of("points", balance));
    }

    private String topMessage(Player player, JsonObject body) {
        var first = DaemonJson.firstObject(body, "players").orElseGet(JsonObject::new);
        var name = DaemonJson.string(first, "name").orElse("-");
        var balance = DaemonJson.integer(first, "balance").map(String::valueOf).orElse("0");
        return message(player, "points.top", Map.of("name", name, "points", balance));
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }
}
