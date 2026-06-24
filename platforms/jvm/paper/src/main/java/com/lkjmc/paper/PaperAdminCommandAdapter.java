package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.permission.PermissionNodes;
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
        if (args.length == 1 && args[0].equalsIgnoreCase("status")) {
            return status(sender);
        }
        if (args.length == 2 && args[0].equalsIgnoreCase("server") && args[1].equalsIgnoreCase("list")) {
            return daemon(sender, PermissionNodes.ADMIN_INSTANCE_LIST, "instance.list", Map.of());
        }
        if (args.length == 3 && args[0].equalsIgnoreCase("server")) {
            return lifecycle(sender, args[1], args[2]);
        }
        if (args.length == 4 && args[0].equalsIgnoreCase("server") && args[1].equalsIgnoreCase("create")) {
            return daemon(sender, PermissionNodes.ADMIN_INSTANCE_CREATE, "instance.create", Map.of(
                "id", args[2], "kind", "paper", "template", args[3]
            ));
        }
        if (args.length == 4 && args[0].equalsIgnoreCase("server") && args[1].equalsIgnoreCase("delete")) {
            if (!args[3].equalsIgnoreCase("confirm")) {
                sender.sendMessage("usage: /lkjmc server delete <id> confirm");
                return true;
            }
            return daemon(sender, PermissionNodes.ADMIN_INSTANCE_DELETE, "instance.delete", Map.of(
                "id", args[2], "force", false
            ));
        }
        sender.sendMessage("usage: /lkjmc status | /lkjmc server ...");
        return true;
    }

    private boolean status(CommandSender sender) {
        if (!sender.hasPermission(PermissionNodes.ADMIN_STATUS)) {
            sender.sendMessage("no permission");
            return true;
        }
        sender.sendMessage("lkjmc paper running; players=" + plugin.getServer().getOnlinePlayers().size());
        send(sender, "status", Map.of());
        return true;
    }

    private boolean lifecycle(CommandSender sender, String action, String id) {
        var command = switch (action) {
            case "start", "stop", "restart" -> "instance." + action;
            default -> "";
        };
        if (command.isBlank()) {
            sender.sendMessage("unknown server action");
            return true;
        }
        return daemon(sender, permission(action), command, Map.of("id", id));
    }

    private boolean daemon(CommandSender sender, String permission, String command, Map<String, Object> body) {
        if (!sender.hasPermission(permission)) {
            sender.sendMessage("no permission");
            return true;
        }
        send(sender, command, body);
        return true;
    }

    private void send(CommandSender sender, String command, Map<String, Object> body) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> reply(sender, response.ok() ? "ok " + command : "failed " + command)),
            () -> sender.sendMessage("daemon unavailable"));
    }

    private void reply(CommandSender sender, String message) {
        if (sender instanceof Player player) {
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(message));
        } else {
            sender.sendMessage(message);
        }
    }

    private static String permission(String action) {
        return switch (action) {
            case "start" -> PermissionNodes.ADMIN_INSTANCE_START;
            case "stop" -> PermissionNodes.ADMIN_INSTANCE_STOP;
            case "restart" -> PermissionNodes.ADMIN_INSTANCE_RESTART;
            default -> PermissionNodes.ADMIN_STATUS;
        };
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
