package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import java.util.Map;
import java.util.UUID;
import org.bukkit.Bukkit;
import org.bukkit.Location;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerJoinEvent;

public final class TeleportArrivalListener implements Listener {
    private final LkjmcPaperPlugin plugin;

    public TeleportArrivalListener(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
    }

    @EventHandler
    public void onJoin(PlayerJoinEvent event) {
        plugin.daemon().ifPresent(client -> client.send(request(event.getPlayer().getUniqueId()))
            .thenAccept(response -> apply(event, response.body().get("raw"))));
    }

    private void apply(PlayerJoinEvent event, Object raw) {
        var json = raw == null ? "" : raw.toString();
        if (!json.contains("\"found\":true")) {
            return;
        }
        var world = Bukkit.getWorld(CrossServerTeleportAdapter.extract(json, "world").orElse("world"));
        if (world == null) {
            return;
        }
        var target = new Location(world, number(json, "x"), number(json, "y"), number(json, "z"));
        target.setYaw((float) number(json, "yaw"));
        target.setPitch((float) number(json, "pitch"));
        plugin.scheduler().runPlayer(event.getPlayer(), () -> event.getPlayer().teleport(target));
    }

    private static DaemonRequest request(UUID playerId) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()),
            "player.teleport.take", Map.of("playerUuid", playerId.toString(), "serverId", instanceId()));
    }

    private static double number(String json, String key) {
        return CrossServerTeleportAdapter.extract(json, key).map(Double::parseDouble).orElse(0.0);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
