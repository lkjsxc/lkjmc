package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Arrays;
import java.util.Map;
import java.util.UUID;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class ModerationCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public ModerationCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (label.equalsIgnoreCase("unban")) {
            return unban(sender, args);
        }
        return ban(sender, args);
    }

    private boolean ban(CommandSender sender, String[] args) {
        if (args.length < 2) {
            sender.sendMessage("usage: /ban <player> <reason>");
            return true;
        }
        var target = plugin.getServer().getPlayerExact(args[0]);
        if (target == null) {
            sender.sendMessage("player unavailable");
            return true;
        }
        var actor = sender instanceof Player player ? player.getName() : "console";
        var reason = String.join(" ", Arrays.copyOfRange(args, 1, args.length));
        call(sender, "player.moderation.ban", Map.of(
            "playerUuid", target.getUniqueId().toString(),
            "playerName", target.getName(),
            "actorName", actor,
            "reason", reason
        ), "moderation.banned");
        return true;
    }

    private boolean unban(CommandSender sender, String[] args) {
        if (args.length != 1) {
            sender.sendMessage("usage: /unban <player>");
            return true;
        }
        call(sender, "player.moderation.unban", Map.of("playerName", args[0]), "moderation.unbanned");
        return true;
    }

    private void call(CommandSender sender, String command, Map<String, Object> body, String okKey) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> reply(sender, response.ok() ? okKey : "moderation.failed")),
            () -> reply(sender, "daemon.unavailable"));
    }

    private void reply(CommandSender sender, String key) {
        if (sender instanceof Player player) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message(player, key)));
            return;
        }
        sender.sendMessage(message(sender, key));
    }

    private String message(CommandSender sender, String key) {
        if (sender instanceof Player player) {
            return renderer.render(player.locale().toLanguageTag(), key, Map.of());
        }
        return renderer.render("en", key, Map.of());
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
