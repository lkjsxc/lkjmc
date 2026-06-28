package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.permission.PermissionNodes;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.Map;
import java.util.UUID;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class EndExpeditionCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public EndExpeditionCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (!player.hasPermission(PermissionNodes.USER_ADVENTURE)) {
            player.sendMessage(message(player, "command.no-permission"));
            return true;
        }
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId()),
            "adventure.end.purchase",
            Map.of(
                "playerUuid", player.getUniqueId().toString(),
                "playerName", player.getName(),
                "acceptMinecraftEula", true
            )
        )).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> handle(player, response.ok(), response.body()))),
            () -> player.sendMessage(message(player, "daemon.unavailable")));
        return true;
    }

    private void handle(Player player, boolean ok, com.google.gson.JsonObject body) {
        if (!ok) {
            player.sendMessage(message(player, "adventure.end.failed"));
            return;
        }
        var target = DaemonJson.string(body, "targetServer").orElse("");
        if (target.isBlank()) {
            player.sendMessage(message(player, "adventure.end.failed"));
            return;
        }
        player.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
            ProfileTransferMessages.transferRequest(target));
        player.sendMessage(message(player, "adventure.end.started"));
    }

    private String message(Player player, String key) {
        return renderer.render(player.locale().toLanguageTag(), key, Map.of());
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
