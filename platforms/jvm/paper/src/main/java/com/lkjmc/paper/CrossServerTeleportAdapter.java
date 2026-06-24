package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.entity.Player;

public final class CrossServerTeleportAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public CrossServerTeleportAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public void request(Player player, String raw, String failureKey) {
        var targetServer = extract(raw, "serverId").orElse("");
        if (targetServer.isBlank()) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message(player, failureKey)));
            return;
        }
        plugin.daemon().ifPresentOrElse(client -> client.send(daemon("player.teleport.request", Map.of(
            "playerUuid", player.getUniqueId().toString(),
            "name", player.getName(),
            "targetServer", targetServer,
            "sourceServer", instanceId(),
            "location", location(raw)
        ))).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> {
            if (response.ok()) {
                player.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
                    ProfileTransferMessages.transferRequest(targetServer));
                player.sendMessage(message(player, "teleport.cross-server"));
            } else {
                player.sendMessage(message(player, failureKey));
            }
        })), () -> player.sendMessage(message(player, "daemon.unavailable")));
    }

    private DaemonRequest daemon(String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body);
    }

    private String message(Player player, String key) {
        return renderer.render(player.locale().toLanguageTag(), key, Map.of());
    }

    private static Map<String, Object> location(String json) {
        return Map.of(
            "world", extract(json, "world").orElse("world"),
            "x", number(json, "x"),
            "y", number(json, "y"),
            "z", number(json, "z"),
            "yaw", number(json, "yaw"),
            "pitch", number(json, "pitch")
        );
    }

    private static double number(String json, String key) {
        return extract(json, key).map(Double::parseDouble).orElse(0.0);
    }

    static Optional<String> extract(String json, String key) {
        var needle = "\"" + key + "\":";
        var start = json.indexOf(needle);
        if (start < 0) {
            return Optional.empty();
        }
        var valueStart = start + needle.length();
        if (valueStart < json.length() && json.charAt(valueStart) == '"') {
            var end = json.indexOf('"', valueStart + 1);
            return end < 0 ? Optional.empty() : Optional.of(json.substring(valueStart + 1, end));
        }
        var end = valueStart;
        while (end < json.length() && "-0123456789.".indexOf(json.charAt(end)) >= 0) {
            end++;
        }
        return end == valueStart ? Optional.empty() : Optional.of(json.substring(valueStart, end));
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
