package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
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
        if (args.length == 2 && args[0].equalsIgnoreCase("invite")) {
            return invite(player, args[1]);
        }
        if (args.length >= 1 && args[0].equalsIgnoreCase("accept")) {
            return send(player, "player.party.accept", Map.of("playerUuid", player.getUniqueId().toString()));
        }
        if (args.length == 1 && args[0].equalsIgnoreCase("info")) {
            return send(player, "player.party.info", Map.of("playerUuid", player.getUniqueId().toString()));
        }
        if (args.length == 1 && args[0].equalsIgnoreCase("leave")) {
            return send(player, "player.party.leave", Map.of("playerUuid", player.getUniqueId().toString()));
        }
        player.sendMessage(message(player, "command.usage", Map.of("usage", "/party create|invite|accept|info|leave")));
        return true;
    }

    private boolean invite(Player player, String name) {
        var target = plugin.getServer().getPlayerExact(name);
        if (target == null || target.getUniqueId().equals(player.getUniqueId())) {
            player.sendMessage(message(player, "party.invite.missing", Map.of()));
            return true;
        }
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "player.party.invite", Map.of(
                "inviterUuid", player.getUniqueId().toString(),
                "inviteeUuid", target.getUniqueId().toString(),
                "inviteeName", target.getName()
            )
        )).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> {
            player.sendMessage(result(player, "player.party.invite", response.ok(), response.body()));
            if (response.ok()) {
                plugin.scheduler().runPlayer(target,
                    () -> target.sendMessage(message(target, "party.invite.received", Map.of("player", player.getName()))));
            }
        })), () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private boolean create(Player player, String name) {
        return send(player, "player.party.create", Map.of(
            "playerUuid", player.getUniqueId().toString(), "playerName", player.getName(), "partyName", name
        ));
    }

    private boolean send(Player player, String command, Map<String, Object> body) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(result(player, command, response.ok(), response.body())))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String result(Player player, String command, boolean ok, JsonObject body) {
        if (!ok) {
            return message(player, "party.failed", Map.of());
        }
        if (command.equals("player.party.info")) {
            if (!DaemonJson.bool(body, "found")) {
                return message(player, "party.none", Map.of());
            }
            return message(player, "party.info", Map.of(
                "name", DaemonJson.string(body, "name").orElse("party"),
                "role", DaemonJson.string(body, "role").orElse("member")
            ));
        }
        return message(player, switch (command) {
            case "player.party.leave" -> "party.left";
            case "player.party.accept" -> "party.joined";
            case "player.party.invite" -> "party.invite.sent";
            default -> "party.created";
        }, Map.of());
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
