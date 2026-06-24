package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.Bukkit;
import org.bukkit.Location;
import org.bukkit.entity.Player;

public final class WarpCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public WarpCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public boolean setWarp(Player player, String[] args) {
        if (args.length != 1) {
            player.sendMessage(message(player, "command.usage", Map.of("usage", "/setwarp <name>")));
            return true;
        }
        plugin.daemon().ifPresentOrElse(client -> client.send(request("player.warp.set", Map.of(
            "warp", args[0],
            "serverId", instanceId(),
            "location", location(player.getLocation())
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
            .thenAccept(response -> applyWarp(player, response.body().get("raw"))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private void applyWarp(Player player, Object raw) {
        var json = raw == null ? "" : raw.toString();
        if (!json.contains("\"found\":true")) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message(player, "warp.not-found", Map.of())));
            return;
        }
        if (!instanceId().equals(extract(json, "serverId").orElse(""))) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message(player, "warp.wrong-server", Map.of())));
            return;
        }
        var world = Bukkit.getWorld(extract(json, "world").orElse("world"));
        if (world == null) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message(player, "warp.failed", Map.of())));
            return;
        }
        var target = new Location(world, number(json, "x"), number(json, "y"), number(json, "z"));
        target.setYaw((float) number(json, "yaw"));
        target.setPitch((float) number(json, "pitch"));
        plugin.scheduler().runPlayer(player, () -> player.teleport(target));
    }

    private DaemonRequest request(String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body);
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
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

    private static double number(String json, String key) {
        return extract(json, key).map(Double::parseDouble).orElse(0.0);
    }

    private static Optional<String> extract(String json, String key) {
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
}
