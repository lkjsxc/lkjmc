package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.UUID;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class KitCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public KitCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (args.length == 0 || (args.length == 1 && args[0].equalsIgnoreCase("list"))) {
            return call(player, "player.kit.list", Map.of(), "kit.list");
        }
        if (args.length == 2 && args[0].equalsIgnoreCase("claim")) {
            return call(player, "player.kit.claim", Map.of(
                "playerUuid", player.getUniqueId().toString(),
                "name", player.getName(),
                "kitId", args[1]
            ), "kit.claim");
        }
        player.sendMessage(message(player, "command.usage", Map.of("usage", "/kit [list|claim <kit>]")));
        return true;
    }

    private boolean call(Player player, String command, Map<String, Object> body, String kind) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(result(player, kind, response.ok(), response.body().get("raw"))))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String result(Player player, String kind, boolean ok, Object raw) {
        if (kind.equals("kit.list")) {
            var count = raw == null ? 0 : count(raw.toString(), "\"id\":");
            return message(player, "kit.list.count", Map.of("count", Integer.toString(count)));
        }
        if (!ok) {
            return message(player, "kit.claim.failed", Map.of());
        }
        var key = raw != null && raw.toString().contains("\"claimed\":true") ? "kit.claimed" : "kit.cooldown";
        return message(player, key, Map.of());
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

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
