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

public final class ReportCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public ReportCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player reporter)) {
            sender.sendMessage("players only");
            return true;
        }
        if (args.length < 2) {
            reporter.sendMessage(message(reporter, "command.usage", Map.of("usage", "/report <player> <reason>")));
            return true;
        }
        var target = plugin.getServer().getPlayerExact(args[0]);
        if (target == null || target.getUniqueId().equals(reporter.getUniqueId())) {
            reporter.sendMessage(message(reporter, "report.missing", Map.of()));
            return true;
        }
        var reason = String.join(" ", Arrays.copyOfRange(args, 1, args.length));
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "player.report.create", Map.of(
                "reporterUuid", reporter.getUniqueId().toString(),
                "reporterName", reporter.getName(),
                "targetUuid", target.getUniqueId().toString(),
                "targetName", target.getName(),
                "serverId", instanceId(),
                "reason", reason
            )
        )).thenAccept(response -> plugin.scheduler().runPlayer(reporter,
            () -> reporter.sendMessage(message(reporter, response.ok() ? "report.sent" : "report.failed", Map.of())))),
            () -> reporter.sendMessage(message(reporter, "daemon.unavailable", Map.of())));
        return true;
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
