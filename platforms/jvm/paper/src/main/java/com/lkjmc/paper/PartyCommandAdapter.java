package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.entity.Player;

public final class PartyCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public PartyCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public boolean handle(Player player, String[] args) {
        if (args.length == 2 && args[0].equalsIgnoreCase("create")) {
            return create(player, args[1]);
        }
        if (args.length == 1 && args[0].equalsIgnoreCase("info")) {
            return send(player, "player.party.info", Map.of("playerUuid", player.getUniqueId().toString()));
        }
        if (args.length == 1 && args[0].equalsIgnoreCase("leave")) {
            return send(player, "player.party.leave", Map.of("playerUuid", player.getUniqueId().toString()));
        }
        player.sendMessage(message(player, "command.usage", Map.of("usage", "/party create <name>|info|leave")));
        return true;
    }

    private boolean create(Player player, String name) {
        return send(player, "player.party.create", Map.of(
            "playerUuid", player.getUniqueId().toString(),
            "playerName", player.getName(),
            "partyName", name
        ));
    }

    private boolean send(Player player, String command, Map<String, Object> body) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(result(player, command, response.ok(), response.body().get("raw"))))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String result(Player player, String command, boolean ok, Object raw) {
        if (!ok) {
            return message(player, "party.failed", Map.of());
        }
        if (command.equals("player.party.info")) {
            var json = raw == null ? "" : raw.toString();
            if (!json.contains("\"found\":true")) {
                return message(player, "party.none", Map.of());
            }
            return message(player, "party.info", Map.of(
                "name", extract(json, "name").orElse("party"),
                "role", extract(json, "role").orElse("member")
            ));
        }
        return message(player, command.equals("player.party.leave") ? "party.left" : "party.created", Map.of());
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
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

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
