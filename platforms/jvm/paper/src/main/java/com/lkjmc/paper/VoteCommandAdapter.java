package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class VoteCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public VoteCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "player.vote.list", Map.of()
        )).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(message(player, response.body().get("raw"))))),
            () -> player.sendMessage(renderer.render(player.locale().toLanguageTag(), "daemon.unavailable", Map.of())));
        return true;
    }

    private String message(Player player, Object raw) {
        var text = raw == null ? "" : raw.toString();
        var count = count(text, "\"id\":");
        if (count == 0) {
            return renderer.render(player.locale().toLanguageTag(), "vote.empty", Map.of());
        }
        return renderer.render(player.locale().toLanguageTag(), "vote.links", Map.of(
            "count", Integer.toString(count), "url", extract(text, "url").orElse("")
        ));
    }

    private static Optional<String> extract(String json, String key) {
        var needle = "\"" + key + "\":\"";
        var start = json.indexOf(needle);
        if (start < 0) {
            return Optional.empty();
        }
        var valueStart = start + needle.length();
        var end = json.indexOf('"', valueStart);
        return end < 0 ? Optional.empty() : Optional.of(json.substring(valueStart, end));
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
