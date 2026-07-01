package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Arrays;
import java.util.Map;
import java.util.UUID;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class WarningCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public WarningCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (label.equalsIgnoreCase("warnings")) {
            return list(sender, args);
        }
        return warn(sender, args);
    }

    private boolean warn(CommandSender sender, String[] args) {
        if (args.length < 2) {
            reply(sender, message(sender, "command.usage", Map.of("usage", "/warn <player> <reason>")));
            return true;
        }
        var target = plugin.getServer().getPlayerExact(args[0]);
        if (target == null) {
            reply(sender, message(sender, "warning.missing", Map.of()));
            return true;
        }
        var actor = sender instanceof Player player ? player.getName() : "console";
        var reason = String.join(" ", Arrays.copyOfRange(args, 1, args.length));
        call(sender, "player.warning.create", Map.of(
            "playerUuid", target.getUniqueId().toString(), "playerName", target.getName(),
            "actorName", actor, "reason", reason
        ), response -> {
            reply(sender, message(sender, response.ok() ? "warning.sent" : "warning.failed", Map.of()));
            if (response.ok()) {
                plugin.scheduler().runPlayer(target,
                    () -> target.sendMessage(message(target, "warning.received", Map.of("reason", reason))));
            }
        });
        return true;
    }

    private boolean list(CommandSender sender, String[] args) {
        if (args.length != 1) {
            reply(sender, message(sender, "command.usage", Map.of("usage", "/warnings <player>")));
            return true;
        }
        var target = plugin.getServer().getPlayerExact(args[0]);
        if (target == null) {
            reply(sender, message(sender, "warning.missing", Map.of()));
            return true;
        }
        call(sender, "player.warning.list", Map.of("playerUuid", target.getUniqueId().toString(), "limit", 10),
            response -> reply(sender, response.ok() ? summary(sender, target.getName(), response.body())
                : message(sender, "warning.failed", Map.of())));
        return true;
    }

    private void call(CommandSender sender, String command, Map<String, Object> body,
        java.util.function.Consumer<com.lkjmc.common.daemon.DaemonResponse> handler) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(handler), () -> reply(sender, message(sender, "daemon.unavailable", Map.of())));
    }

    private String summary(CommandSender sender, String player, JsonObject body) {
        var count = DaemonJson.arraySize(body, "warnings");
        if (count == 0) {
            return message(sender, "warnings.empty", Map.of("player", player));
        }
        var id = DaemonJson.firstObject(body, "warnings").flatMap(item -> DaemonJson.string(item, "id")).orElse("");
        return message(sender, "warnings.count", Map.of("player", player, "count", Integer.toString(count), "id", id));
    }

    private void reply(CommandSender sender, String text) {
        if (sender instanceof Player player) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(text));
            return;
        }
        sender.sendMessage(text);
    }

    private String message(CommandSender sender, String key, Map<String, String> values) {
        return renderer.render(sender instanceof Player player ? plugin.localeService().locale(player) : "en", key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
