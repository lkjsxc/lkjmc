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

public final class NoteCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public NoteCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        return label.equalsIgnoreCase("notes") ? list(sender, args) : note(sender, args);
    }

    private boolean note(CommandSender sender, String[] args) {
        if (args.length < 2) {
            reply(sender, message(sender, "command.usage", Map.of("usage", "/note <player> <note>")));
            return true;
        }
        var target = plugin.getServer().getPlayerExact(args[0]);
        if (target == null) {
            reply(sender, message(sender, "note.missing", Map.of()));
            return true;
        }
        var actor = sender instanceof Player player ? player.getName() : "console";
        var body = String.join(" ", Arrays.copyOfRange(args, 1, args.length));
        call(sender, "player.note.create", Map.of(
            "playerUuid", target.getUniqueId().toString(),
            "playerName", target.getName(), "actorName", actor, "body", body
        ), ok -> message(sender, ok ? "note.saved" : "note.failed", Map.of()));
        return true;
    }

    private boolean list(CommandSender sender, String[] args) {
        if (args.length != 1) {
            reply(sender, message(sender, "command.usage", Map.of("usage", "/notes <player>")));
            return true;
        }
        var target = plugin.getServer().getPlayerExact(args[0]);
        if (target == null) {
            reply(sender, message(sender, "note.missing", Map.of()));
            return true;
        }
        call(sender, "player.note.list", Map.of("playerUuid", target.getUniqueId().toString(), "limit", 10),
            ok -> message(sender, ok ? "notes.listed" : "note.failed", Map.of("player", target.getName())));
        return true;
    }

    private void call(CommandSender sender, String command, Map<String, Object> body,
        java.util.function.Function<Boolean, String> formatter) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> reply(sender, formatter.apply(response.ok()))),
            () -> reply(sender, message(sender, "daemon.unavailable", Map.of())));
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
            return renderer.render(plugin.localeService().locale(player), key, values);
        }
        return renderer.render("en", key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
