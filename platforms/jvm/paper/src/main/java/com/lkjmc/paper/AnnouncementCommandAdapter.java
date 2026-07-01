package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.ArrayList;
import java.util.Map;
import java.util.UUID;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class AnnouncementCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public AnnouncementCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (args.length == 0) {
            reply(sender, message(sender, "command.usage", Map.of("usage", "/announce <message>")));
            return true;
        }
        var text = String.join(" ", args);
        var recipients = new ArrayList<Player>();
        recipients.addAll(plugin.getServer().getOnlinePlayers());
        var actor = sender instanceof Player player ? player.getName() : "console";
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId()),
            "announcement.create",
            Map.of("actorName", actor, "serverId", instanceId(), "message", text)
        )).thenAccept(response -> {
            if (!response.ok()) {
                reply(sender, message(sender, "announcement.failed", Map.of()));
                return;
            }
            broadcast(recipients, text);
            reply(sender, message(sender, "announcement.sent", Map.of()));
        }), () -> reply(sender, message(sender, "daemon.unavailable", Map.of())));
        return true;
    }

    private void broadcast(Iterable<Player> players, String text) {
        for (var player : players) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message(player, "announcement.broadcast", Map.of("message", text))));
        }
    }

    private void reply(CommandSender sender, String text) {
        if (sender instanceof Player player) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(text));
            return;
        }
        sender.sendMessage(text);
    }

    private String message(CommandSender sender, String key, Map<String, String> values) {
        if (sender instanceof Player player) {
            return renderer.render(plugin.localeService().locale(player), key, values);
        }
        return renderer.render("en", key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
