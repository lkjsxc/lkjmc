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

public final class MailCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public MailCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (args.length == 0 || args[0].equalsIgnoreCase("inbox")) {
            return inbox(player);
        }
        if (args[0].equalsIgnoreCase("read") && args.length == 2) {
            return read(player, args[1]);
        }
        if (args[0].equalsIgnoreCase("send") && args.length >= 3) {
            return send(player, args[1], String.join(" ", Arrays.copyOfRange(args, 2, args.length)));
        }
        player.sendMessage(message(player, "command.usage", Map.of("usage", "/mail inbox|read <id>|send <player> <message>")));
        return true;
    }

    private boolean inbox(Player player) {
        return call(player, "player.mail.inbox", Map.of("playerUuid", player.getUniqueId().toString(), "limit", 10), body -> {
            var count = DaemonJson.arraySize(body, "messages");
            player.sendMessage(message(player, "mail.inbox.count", Map.of("count", Integer.toString(count))));
        });
    }

    private boolean read(Player player, String id) {
        return call(player, "player.mail.read", Map.of(
            "playerUuid", player.getUniqueId().toString(), "messageId", id
        ), body -> {
            if (!DaemonJson.bool(body, "found")) {
                player.sendMessage(message(player, "mail.not-found", Map.of()));
                return;
            }
            player.sendMessage(message(player, "mail.read", Map.of(
                "sender", DaemonJson.string(body, "senderName").orElse(""),
                "body", DaemonJson.string(body, "body").orElse("")
            )));
        });
    }

    private boolean send(Player player, String target, String body) {
        return call(player, "player.mail.send", Map.of(
            "playerUuid", player.getUniqueId().toString(),
            "senderName", player.getName(),
            "recipientName", target,
            "message", body
        ), responseBody -> player.sendMessage(message(player, "mail.sent", Map.of())));
    }

    private boolean call(Player player, String command, Map<String, Object> body,
        java.util.function.Consumer<JsonObject> ok) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> {
            if (response.ok()) {
                ok.accept(response.body());
            } else {
                player.sendMessage(message(player, "mail.failed", Map.of()));
            }
        })), () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
