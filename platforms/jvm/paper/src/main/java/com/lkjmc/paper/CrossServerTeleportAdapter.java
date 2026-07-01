package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.Map;
import java.util.UUID;
import org.bukkit.entity.Player;

public final class CrossServerTeleportAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public CrossServerTeleportAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public void request(Player player, JsonObject body, String failureKey) {
        var targetServer = DaemonJson.string(body, "serverId").orElse("");
        if (targetServer.isBlank()) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message(player, failureKey)));
            return;
        }
        plugin.daemon().ifPresentOrElse(client -> client.send(daemon("player.teleport.request", Map.of(
            "playerUuid", player.getUniqueId().toString(),
            "name", player.getName(),
            "targetServer", targetServer,
            "sourceServer", instanceId(),
            "location", location(body)
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

    static Map<String, Object> location(JsonObject body) {
        var location = DaemonJson.object(body, "location").orElseGet(JsonObject::new);
        return Map.of(
            "world", DaemonJson.string(location, "world").orElse("world"),
            "x", DaemonJson.decimal(location, "x").orElse(0.0),
            "y", DaemonJson.decimal(location, "y").orElse(0.0),
            "z", DaemonJson.decimal(location, "z").orElse(0.0),
            "yaw", DaemonJson.decimal(location, "yaw").orElse(0.0),
            "pitch", DaemonJson.decimal(location, "pitch").orElse(0.0)
        );
    }

    static double locationNumber(JsonObject body, String key) {
        return DaemonJson.object(body, "location")
            .flatMap(location -> DaemonJson.decimal(location, key))
            .orElse(0.0);
    }

    static String locationString(JsonObject body, String key, String fallback) {
        return DaemonJson.object(body, "location")
            .flatMap(location -> DaemonJson.string(location, key))
            .orElse(fallback);
    }

    private DaemonRequest daemon(String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body);
    }

    private String message(Player player, String key) {
        return renderer.render(plugin.localeService().locale(player), key, Map.of());
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
