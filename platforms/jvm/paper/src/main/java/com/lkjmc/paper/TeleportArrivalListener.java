package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
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
            .thenAccept(response -> apply(event, response.body())));
    }

    private void apply(PlayerJoinEvent event, JsonObject body) {
        if (!DaemonJson.bool(body, "found")) {
            return;
        }
        var world = Bukkit.getWorld(CrossServerTeleportAdapter.locationString(body, "world", "world"));
        if (world == null) {
            return;
        }
        var target = new Location(world, number(body, "x"), number(body, "y"), number(body, "z"));
        target.setYaw((float) number(body, "yaw"));
        target.setPitch((float) number(body, "pitch"));
        plugin.scheduler().runPlayer(event.getPlayer(), () -> event.getPlayer().teleport(target));
    }

    private static DaemonRequest request(UUID playerId) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()),
            "player.teleport.take", Map.of("playerUuid", playerId.toString(), "serverId", instanceId()));
    }

    private static double number(JsonObject body, String key) {
        return CrossServerTeleportAdapter.locationNumber(body, key);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
