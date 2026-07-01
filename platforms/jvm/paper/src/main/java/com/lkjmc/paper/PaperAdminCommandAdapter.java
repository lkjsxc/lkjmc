package com.lkjmc.paper;
import com.google.gson.JsonObject;
import com.lkjmc.common.command.CommandInvocation;
import com.lkjmc.common.command.CommandPlatform;
import com.lkjmc.common.command.LkjmcCommandTree;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonHttpConfigStatus;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.permission.PermissionNodes;
import com.lkjmc.common.permission.PrincipalIdentity;
import java.util.HashMap;
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
        if (!parsed.success()) {
            sender.sendMessage("usage: " + parsed.usage());
            return true;
        }
        return execute(sender, parsed.invocation());
    }
    private boolean execute(CommandSender sender, CommandInvocation invocation) {
        if (!allowed(sender, invocation.spec().permission())) {
            sender.sendMessage("no permission: " + invocation.spec().permission());
            return true;
        }
        switch (invocation.spec().target()) {
            case "status" -> status(sender);
            case "doctor", "config.check" -> doctor(sender);
            case "config.reload" -> daemon(sender, "config.reload", Map.of());
            case "restart.warn" -> warn(sender, invocation.argument("seconds"));
            case "instance.list" -> daemon(sender, "instance.list", Map.of());
            case "instance.create" -> daemon(sender, "instance.create", Map.of(
                "id", invocation.argument("server"), "kind", kind(invocation.argument("template")),
                "template", invocation.argument("template"), "acceptMinecraftEula", true));
            case "instance.delete" -> daemon(sender, "instance.delete", Map.of(
                "id", invocation.argument("server"), "force", false));
            case "instance.start", "instance.stop", "instance.restart" -> daemon(sender,
                invocation.spec().target(), Map.of("id", invocation.argument("server")));
            case "admin.role.list", "security.daemon-token.status", "security.daemon-token.rotate",
                "economy.catalog.seed-defaults", "adventure.catalog.list", "adventure.session.list" ->
                daemon(sender, invocation.spec().target(), Map.of());
            case "admin.grant.create", "admin.grant.revoke" -> daemon(sender,
                invocation.spec().target(), grantBody(invocation));
            case "admin.principal.inspect" -> daemon(sender, "admin.principal.inspect",
                principalBody(invocation.argument("principal")));
            case "admin.audit.tail" -> daemon(sender, "admin.audit.tail",
                Map.of("lines", Integer.parseInt(invocation.argument("lines"))));
            case "adventure.purchase" -> daemon(sender, "adventure.purchase",
                adventureBody(sender, invocation.argument("adventure")));
            case "adventure.return" -> sender.sendMessage("Use /endexpedition return from a temporary backend.");
            case "adventure.session.cancel" -> daemon(sender, "adventure.session.cancel", Map.of(
                "sessionId", invocation.argument("session"), "reason", invocation.argument("reason")));
            default -> sender.sendMessage("unsupported on Paper: " + invocation.spec().usage());
        }
        return true;
    }
    private Map<String, Object> grantBody(CommandInvocation invocation) {
        var body = new HashMap<String, Object>(principalBody(invocation.argument("principal")));
        body.put("roleId", invocation.argument("role"));
        body.put("reason", invocation.argument("reason"));
        return body;
    }
    private Map<String, Object> principalBody(String principal) {
        var parts = principal.split(":", 2);
        var kind = parts.length == 2 ? parts[0] : "minecraft-player";
        var id = parts.length == 2 ? parts[1] : principal;
        return Map.of("subjectKind", kind, "subjectId", id);
    }
    private Map<String, Object> adventureBody(CommandSender sender, String adventureId) {
        if (sender instanceof Player player) {
            return Map.of("adventureId", adventureId, "playerUuid", player.getUniqueId().toString(),
                "playerName", player.getName(), "acceptMinecraftEula", true);
        }
        return Map.of("adventureId", adventureId);
    }
    private String kind(String template) {
        if (template.startsWith("folia")) return "folia";
        if (template.startsWith("purpur")) return "purpur";
        if (template.startsWith("velocity")) return "velocity";
        return "paper";
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
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, principal(sender, command, body)
        )).thenAccept(response -> reply(sender, format(command, response.ok(), response.body(),
            response.error().map(error -> error.code()).orElse("daemon.command_failed")))),
            () -> sender.sendMessage("daemon unavailable: " + DaemonHttpConfigStatus.fromEnv().code()));
    }
    private Map<String, Object> principal(CommandSender sender, String command, Map<String, Object> body) {
        var values = new HashMap<String, Object>(body);
        values.put("platformPermission", platformAllowed(sender, permission(command)));
        if (sender instanceof Player player) {
            values.put("principalKind", "minecraft-player");
            values.put("principalId", player.getUniqueId().toString());
            values.put("principalName", player.getName());
        }
        return values;
    }
    private boolean allowed(CommandSender sender, String permission) {
        var platform = platformAllowed(sender, permission);
        if (sender instanceof Player player && plugin != null) {
            return plugin.adminGrants().decide(identity(player), permission, platform, player.isOp()).allowed();
        }
        return platform;
    }
    private boolean platformAllowed(CommandSender sender, String permission) {
        if (sender instanceof Player player) {
            return sender.hasPermission(permission) || player.isOp();
        }
        return sender.hasPermission(permission);
    }
    private PrincipalIdentity identity(Player player) {
        return new PrincipalIdentity("minecraft-player", player.getUniqueId().toString(), player.getName());
    }
    private String permission(String command) {
        return switch (command) {
            case "status", "doctor" -> PermissionNodes.ADMIN_STATUS;
            case "config.reload" -> PermissionNodes.ADMIN_RELOAD;
            case "admin.role.list", "admin.grant.create", "admin.grant.revoke", "admin.principal.inspect",
                "admin.audit.tail", "security.daemon-token.status", "security.daemon-token.rotate" ->
                PermissionNodes.ADMIN_ADMIN;
            case "economy.catalog.seed-defaults" -> PermissionNodes.ADMIN_ECONOMY;
            case "adventure.catalog.list", "adventure.purchase", "adventure.return" -> PermissionNodes.USER_ADVENTURE;
            case "adventure.session.list" -> PermissionNodes.ADMIN_INSTANCE_LIST;
            case "adventure.session.cancel" -> PermissionNodes.ADMIN_INSTANCE_DELETE;
            case "instance.list" -> PermissionNodes.ADMIN_INSTANCE_LIST;
            case "instance.create" -> PermissionNodes.ADMIN_INSTANCE_CREATE;
            case "instance.start" -> PermissionNodes.ADMIN_INSTANCE_START;
            case "instance.stop" -> PermissionNodes.ADMIN_INSTANCE_STOP;
            case "instance.restart" -> PermissionNodes.ADMIN_INSTANCE_RESTART;
            case "instance.delete" -> PermissionNodes.ADMIN_INSTANCE_DELETE;
            default -> "lkjmc.admin.status";
        };
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
