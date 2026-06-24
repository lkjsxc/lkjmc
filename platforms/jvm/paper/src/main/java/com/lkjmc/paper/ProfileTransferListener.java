package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.Map;
import java.util.UUID;
import org.bukkit.Bukkit;
import org.bukkit.Location;
import org.bukkit.entity.Player;
import org.bukkit.plugin.messaging.PluginMessageListener;

public final class ProfileTransferListener implements PluginMessageListener {
    private final LkjmcPaperPlugin plugin;
    private final PlayerProfileAdapter profiles = new PlayerProfileAdapter();

    public ProfileTransferListener(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
    }

    @Override
    public void onPluginMessageReceived(String channel, Player player, byte[] message) {
        if (!channel.equals(ProfileTransferMessages.CHANNEL) || plugin.daemon().isEmpty()) {
            return;
        }
        ProfileTransferMessages.parse("save", message).ifPresent(requestId ->
            plugin.scheduler().runPlayer(player, () -> saveAndAck(player, requestId)));
        ProfileTransferMessages.parseText("arrive", message).ifPresent(location ->
            plugin.scheduler().runPlayer(player, () -> teleport(player, location)));
    }

    private void saveAndAck(Player player, UUID requestId) {
        var instanceId = System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
        var snapshot = profiles.capture(player);
        plugin.daemon().get().send(request(instanceId, "player.snapshot", Map.of(
            "playerUuid", player.getUniqueId().toString(),
            "name", player.getName(),
            "sourceInstance", instanceId,
            "scope", "profile",
            "payloadBase64", snapshot.payloadBase64(),
            "sha256", snapshot.sha256()
        ))).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> {
            if (response.ok()) {
                player.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
                    ProfileTransferMessages.saved(requestId));
                plugin.daemon().get().send(request(instanceId, "player.transfer.saved", Map.of(
                    "playerUuid", player.getUniqueId().toString()
                )));
            }
        }));
    }

    private void teleport(Player player, String encoded) {
        var parts = encoded.split("\\|", -1);
        if (parts.length != 6) {
            return;
        }
        var world = Bukkit.getWorld(parts[0]);
        if (world == null) {
            return;
        }
        var target = new Location(world, parse(parts[1]), parse(parts[2]), parse(parts[3]));
        target.setYaw((float) parse(parts[4]));
        target.setPitch((float) parse(parts[5]));
        player.teleport(target);
    }

    private static double parse(String value) {
        try {
            return Double.parseDouble(value);
        } catch (NumberFormatException error) {
            return 0.0;
        }
    }

    private static DaemonRequest request(String instanceId, String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId), command, body);
    }
}
