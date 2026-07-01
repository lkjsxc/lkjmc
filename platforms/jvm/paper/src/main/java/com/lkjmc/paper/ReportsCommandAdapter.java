package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.UUID;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class ReportsCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public ReportsCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (args.length == 0) {
            return list(sender);
        }
        if (args.length == 2 && (args[0].equalsIgnoreCase("resolve") || args[0].equalsIgnoreCase("dismiss"))) {
            return close(sender, args[0].toLowerCase(), args[1]);
        }
        reply(sender, message(sender, "command.usage", Map.of("usage", "/reports [resolve|dismiss <id>]")));
        return true;
    }

    private boolean list(CommandSender sender) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "player.report.list", Map.of("limit", 10)
        )).thenAccept(response -> reply(sender, response.ok() ? summary(sender, response.body())
            : message(sender, "reports.failed", Map.of()))),
            () -> reply(sender, message(sender, "daemon.unavailable", Map.of())));
        return true;
    }

    private boolean close(CommandSender sender, String action, String reportId) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "player.report." + action,
            Map.of("reportId", reportId)
        )).thenAccept(response -> {
            var key = response.ok() && DaemonJson.integer(response.body(), "closed").orElse(0L) == 1L
                ? "reports.closed" : "reports.close.failed";
            reply(sender, message(sender, key, Map.of()));
        }), () -> reply(sender, message(sender, "daemon.unavailable", Map.of())));
        return true;
    }

    private String summary(CommandSender sender, JsonObject body) {
        var count = DaemonJson.arraySize(body, "reports");
        if (count == 0) {
            return message(sender, "reports.empty", Map.of());
        }
        var id = DaemonJson.firstObject(body, "reports").flatMap(item -> DaemonJson.string(item, "id")).orElse("");
        return message(sender, "reports.count", Map.of("count", Integer.toString(count), "id", id));
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
