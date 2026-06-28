package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.command.CommandInvocation;
import com.lkjmc.common.command.CommandPlatform;
import com.lkjmc.common.command.LkjmcCommandTree;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonHttpConfigStatus;
import com.lkjmc.common.daemon.DaemonRequest;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class PaperAdminCommandAdapter {
    private final LkjmcPaperPlugin plugin;

    public PaperAdminCommandAdapter(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
    }

    public boolean handle(CommandSender sender, String[] args) {
        var parsed = LkjmcCommandTree.parse(CommandPlatform.PAPER, List.of(args));
        if (parsed.isEmpty()) {
            sender.sendMessage("usage: " + LkjmcCommandTree.usage(CommandPlatform.PAPER, List.of(args)));
            return true;
        }
        return execute(sender, parsed.get());
    }

    private boolean execute(CommandSender sender, CommandInvocation invocation) {
        if (!sender.hasPermission(invocation.spec().permission())) {
            sender.sendMessage("no permission: " + invocation.spec().permission());
            return true;
        }
        switch (invocation.spec().target()) {
            case "status" -> status(sender);
            case "doctor" -> doctor(sender);
            case "config.reload" -> daemon(sender, "config.reload", Map.of());
            case "restart.warn" -> warn(sender, invocation.argument("seconds"));
            case "instance.list" -> daemon(sender, "instance.list", Map.of());
            case "instance.create" -> daemon(sender, "instance.create", Map.of(
                "id", invocation.argument("server"), "kind", "paper", "template", invocation.argument("template")));
            case "instance.delete" -> daemon(sender, "instance.delete", Map.of(
                "id", invocation.argument("server"), "force", false));
            case "instance.start", "instance.stop", "instance.restart" -> daemon(sender,
                invocation.spec().target(), Map.of("id", invocation.argument("server")));
            default -> sender.sendMessage("unsupported on Paper: " + invocation.spec().usage());
        }
        return true;
    }

    private void status(CommandSender sender) {
        sender.sendMessage("lkjmc paper running; players=" + plugin.getServer().getOnlinePlayers().size());
        send(sender, "status", Map.of());
    }

    private void doctor(CommandSender sender) {
        var config = DaemonHttpConfigStatus.fromEnv();
        sender.sendMessage("lkjmc doctor: platform=paper root=/lkjmc");
        sender.sendMessage("daemon http: " + config.code());
        if (config.configured()) {
            send(sender, "doctor", Map.of());
        }
    }

    private void warn(CommandSender sender, String seconds) {
        try {
            var value = Integer.parseInt(seconds);
            plugin.getServer().broadcastMessage("lkjmc restart warning: " + value + "s");
            sender.sendMessage("ok restart warning");
        } catch (NumberFormatException error) {
            sender.sendMessage("usage: /lkjmc restart warn <seconds>");
        }
    }

    private void daemon(CommandSender sender, String command, Map<String, Object> body) {
        send(sender, command, body);
    }

    private void send(CommandSender sender, String command, Map<String, Object> body) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> reply(sender, format(command, response.ok(), response.body(),
            response.error().map(error -> error.code()).orElse("daemon.command_failed")))),
            () -> sender.sendMessage("daemon unavailable: " + DaemonHttpConfigStatus.fromEnv().code()));
    }

    private String format(String command, boolean ok, JsonObject body, String error) {
        if (!ok) {
            return "failed " + command + ": " + error;
        }
        if (command.equals("instance.list") && body.has("instances") && body.get("instances").isJsonArray()) {
            var names = new java.util.ArrayList<String>();
            for (var value : body.getAsJsonArray("instances")) {
                if (value.isJsonObject() && value.getAsJsonObject().has("id")) {
                    names.add(value.getAsJsonObject().get("id").getAsString());
                }
            }
            return names.isEmpty() ? "servers: none" : "servers: " + String.join(", ", names);
        }
        return "ok " + command;
    }

    private void reply(CommandSender sender, String message) {
        if (sender instanceof Player player) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message));
        } else {
            sender.sendMessage(message);
        }
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
