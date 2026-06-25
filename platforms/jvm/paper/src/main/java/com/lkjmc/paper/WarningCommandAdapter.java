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
            "playerUuid", target.getUniqueId().toString(),
            "playerName", target.getName(),
            "actorName", actor,
            "reason", reason
        ), response -> {
            reply(sender, message(sender, response.ok() ? "warning.sent" : "warning.failed", Map.of()));
            if (response.ok()) {
                plugin.scheduler().runPlayer(target, () -> target.sendMessage(message(target, "warning.received", Map.of("reason", reason))));
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
        call(sender, "player.warning.list", Map.of(
            "playerUuid", target.getUniqueId().toString(),
            "limit", 10
        ), response -> {
            var raw = response.body().getOrDefault("raw", "").toString();
            reply(sender, response.ok() ? summary(sender, target.getName(), raw)
                : message(sender, "warning.failed", Map.of()));
        });
        return true;
    }

    private void call(CommandSender sender, String command, Map<String, Object> body,
        java.util.function.Consumer<com.lkjmc.common.daemon.DaemonResponse> handler) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(handler), () -> reply(sender, message(sender, "daemon.unavailable", Map.of())));
    }

    private String summary(CommandSender sender, String player, String raw) {
        var count = count(raw, "\"id\":");
        if (count == 0) {
            return message(sender, "warnings.empty", Map.of("player", player));
        }
        return message(sender, "warnings.count", Map.of(
            "player", player, "count", Integer.toString(count), "id", extract(raw, "id")));
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
            return renderer.render(player.locale().toLanguageTag(), key, values);
        }
        return renderer.render("en", key, values);
    }

    private static String extract(String json, String key) {
        var needle = "\"" + key + "\":\"";
        var start = json.indexOf(needle);
        if (start < 0) {
            return "";
        }
        var valueStart = start + needle.length();
        var end = json.indexOf('"', valueStart);
        return end < 0 ? "" : json.substring(valueStart, end);
    }

    private static int count(String value, String needle) {
        var count = 0;
        var index = value.indexOf(needle);
        while (index >= 0) {
            count++;
            index = value.indexOf(needle, index + needle.length());
        }
        return count;
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
