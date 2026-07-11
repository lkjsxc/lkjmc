package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.UUID;
import org.bukkit.Bukkit;
import org.bukkit.Location;
import org.bukkit.entity.Player;

public final class WarpCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final CrossServerTeleportAdapter crossServer;

    public WarpCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
        this.crossServer = new CrossServerTeleportAdapter(plugin, renderer);
    }

    public boolean setWarp(Player player, String[] args) {
        if (args.length != 1) {
            player.sendMessage(message(player, "command.usage", Map.of("usage", "/setwarp <name>")));
            return true;
        }
        plugin.daemon().ifPresentOrElse(client -> client.send(request("player.warp.set", Map.of(
            "warp", args[0], "serverId", instanceId(), "location", location(player.getLocation())
        ))).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(message(player, response.ok() ? "warp.saved" : "warp.failed", Map.of())))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    public boolean warp(Player player, String[] args) {
        if (args.length != 1) {
            player.sendMessage(message(player, "command.usage", Map.of("usage", "/warp <name>")));
            return true;
        }
        plugin.daemon().ifPresentOrElse(client -> client.send(request("player.warp.get", Map.of("warp", args[0])))
            .thenAccept(response -> applyWarp(player, response.body())),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private void applyWarp(Player player, JsonObject body) {
        plugin.scheduler().runGlobal(() -> applyWarpGlobal(player, body));
    }

    private void applyWarpGlobal(Player player, JsonObject body) {
        if (!DaemonJson.bool(body, "found")) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message(player, "warp.not-found", Map.of())));
            return;
        }
        if (!instanceId().equals(DaemonJson.string(body, "serverId").orElse(""))) {
            plugin.scheduler().runPlayer(player, () -> crossServer.request(player, body, "warp.wrong-server"));
            return;
        }
        var world = Bukkit.getWorld(CrossServerTeleportAdapter.locationString(body, "world", "world"));
        if (world == null) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message(player, "warp.failed", Map.of())));
            return;
        }
        var target = new Location(world, number(body, "x"), number(body, "y"), number(body, "z"));
        target.setYaw((float) number(body, "yaw"));
        target.setPitch((float) number(body, "pitch"));
        plugin.scheduler().runPlayer(player, () -> player.teleportAsync(target));
    }

    private DaemonRequest request(String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body);
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }

    private String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }

    private static Map<String, Object> location(Location location) {
        return Map.of(
            "world", location.getWorld().getName(),
            "x", location.getX(),
            "y", location.getY(),
            "z", location.getZ(),
            "yaw", location.getYaw(),
            "pitch", location.getPitch()
        );
    }

    private static double number(JsonObject body, String key) {
        return CrossServerTeleportAdapter.locationNumber(body, key);
    }
}
