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
        if (args.length > 1) {
            player.sendMessage(renderer.render(player.locale().toLanguageTag(), "command.usage", Map.of("usage", "/vote [id]")));
            return true;
        }
        var selected = args.length == 1 ? args[0] : null;
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "player.vote.list", Map.of()
        )).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(message(player, response.body(), selected)))),
            () -> player.sendMessage(renderer.render(player.locale().toLanguageTag(), "daemon.unavailable", Map.of())));
        return true;
    }

    private String message(Player player, JsonObject body, String selected) {
        var count = DaemonJson.arraySize(body, "links");
        if (count == 0) {
            return renderer.render(player.locale().toLanguageTag(), "vote.empty", Map.of());
        }
        if (selected != null) {
            return selected(player, body, selected);
        }
        var url = DaemonJson.firstObject(body, "links").flatMap(item -> DaemonJson.string(item, "url")).orElse("");
        return renderer.render(player.locale().toLanguageTag(), "vote.links", Map.of(
            "count", Integer.toString(count), "url", url
        ));
    }

    private String selected(Player player, JsonObject body, String selected) {
        for (var item : body.getAsJsonArray("links")) {
            if (item.isJsonObject() && selected.equals(DaemonJson.string(item.getAsJsonObject(), "id").orElse(""))) {
                var url = DaemonJson.string(item.getAsJsonObject(), "url").orElse("");
                return renderer.render(player.locale().toLanguageTag(), "vote.selected", Map.of("url", url));
            }
        }
        return renderer.render(player.locale().toLanguageTag(), "vote.not-found", Map.of());
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
