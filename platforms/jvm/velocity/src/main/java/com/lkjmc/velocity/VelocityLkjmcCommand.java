package com.lkjmc.velocity;
import com.google.gson.JsonObject;
import com.lkjmc.common.command.CommandCompletionContext;
import com.lkjmc.common.command.CommandInvocation;
import com.lkjmc.common.command.CommandPlatform;
import com.lkjmc.common.command.LkjmcCommandTree;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonHttpConfigStatus;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.permission.PermissionSnapshotCache;
import com.lkjmc.common.permission.PrincipalIdentity;
import com.velocitypowered.api.command.CommandSource;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;
final class VelocityLkjmcCommand {
    private final ProxyServer proxy;
    private final Optional<DaemonClient> daemon;
    private final Optional<VelocityServerRegistry> registry;
    private final VelocityRestartAdapter restart;
    private final VelocitySendAdapter send;
    private final PermissionSnapshotCache adminGrants;
    private final VelocityTemporarySendAdapter temporarySend;
    private final VelocityWakeJoinAdapter wakeJoin;
    VelocityLkjmcCommand(ProxyServer proxy, Optional<DaemonClient> daemon,
                         Optional<VelocityServerRegistry> registry,
                         VelocityRestartAdapter restart, ProfileSaveBridge transfers) {
        this(proxy, daemon, registry, restart, transfers, PermissionSnapshotCache.disabled());
    }
    VelocityLkjmcCommand(ProxyServer proxy, Optional<DaemonClient> daemon,
                         Optional<VelocityServerRegistry> registry,
                         VelocityRestartAdapter restart, ProfileSaveBridge transfers,
                         PermissionSnapshotCache adminGrants) {
        this.proxy = proxy;
        this.daemon = daemon == null ? Optional.empty() : daemon;
        this.registry = registry == null ? Optional.empty() : registry;
        this.restart = restart;
        this.send = new VelocitySendAdapter(proxy, transfers);
        this.adminGrants = adminGrants == null ? PermissionSnapshotCache.disabled() : adminGrants;
        this.temporarySend = new VelocityTemporarySendAdapter(proxy, this.daemon, send);
        this.wakeJoin = new VelocityWakeJoinAdapter(proxy, this.daemon, this.registry, send);
    }
    int execute(CommandSource source, List<String> args) {
        var parsed = LkjmcCommandTree.parse(CommandPlatform.VELOCITY, args);
        if (!parsed.success()) {
            usage(source, args);
            return 1;
        }
        execute(source, parsed.invocation());
        return 1;
    }
    int usage(CommandSource source, List<String> args) {
        message(source, "usage: " + LkjmcCommandTree.usage(CommandPlatform.VELOCITY, args), NamedTextColor.YELLOW);
        return 1;
    }
    boolean hasAnyPermission(CommandSource source) {
        return LkjmcCommandTree.specs().stream()
            .filter(spec -> spec.supports(CommandPlatform.VELOCITY))
            .anyMatch(spec -> hasPermission(source, spec.permission()));
    }
    boolean canUsePrefix(CommandSource source, List<String> pathPrefix) {
        return LkjmcCommandTree.specs().stream()
            .filter(spec -> spec.supports(CommandPlatform.VELOCITY))
            .filter(spec -> spec.path().size() >= pathPrefix.size())
            .filter(spec -> spec.path().subList(0, pathPrefix.size()).equals(pathPrefix))
            .anyMatch(spec -> hasPermission(source, spec.permission()));
    }
    List<String> suggest(CommandSource source, List<String> args) {
        return LkjmcCommandTree.suggest(CommandPlatform.VELOCITY, args,
            permission -> hasPermission(source, permission), context());
    }
    CommandCompletionContext context() {
        var servers = proxy.getAllServers().stream().map(server -> server.getServerInfo().getName()).sorted().toList();
        var players = proxy.getAllPlayers().stream().map(player -> player.getUsername()).sorted().toList();
        return new CommandCompletionContext(servers, players, List.of("paper", "folia", "purpur"));
    }
    private boolean hasPermission(CommandSource source, String permission) {
        var platform = source.hasPermission(permission);
        if (source instanceof Player player) {
            return adminGrants.decide(identity(player), permission, platform, false).allowed();
        }
        return platform;
    }
    private PrincipalIdentity identity(Player player) {
        return new PrincipalIdentity("minecraft-player", player.getUniqueId().toString(), player.getUsername());
    }
    private void execute(CommandSource source, CommandInvocation command) {
        if (!hasPermission(source, command.spec().permission())) {
            message(source, "no permission: " + command.spec().permission(), NamedTextColor.RED);
            return;
        }
        switch (command.spec().target()) {
            case "status" -> status(source);
            case "doctor", "config.check" -> doctor(source);
            case "config.reload" -> reload(source);
            case "restart.warn" -> warnRestart(source, command.argument("seconds"));
            case "proxy.send" -> send.send(source, command.argument("player"), command.argument("server"));
            case "temporary.send" -> temporarySend.send(source, command.argument("player"), command.argument("instance"));
            case "wake.send" -> wakeJoin.send(source, command.argument("player"), command.argument("server"));
            case "instance.list" -> sendServerList(source);
            case "instance.create" -> sendDaemon(source, "instance.create", Map.of(
                "id", command.argument("server"), "kind", "paper", "template", command.argument("template")));
            case "instance.delete" -> sendDaemon(source, "instance.delete", Map.of(
                "id", command.argument("server"), "force", false));
            case "instance.start", "instance.stop", "instance.restart" -> sendDaemon(source,
                command.spec().target(), Map.of("id", command.argument("server")));
            case "admin.role.list", "security.daemon-token.status", "security.daemon-token.rotate",
                "economy.catalog.seed-defaults", "adventure.catalog.list", "adventure.session.list" ->
                sendDaemon(source, command.spec().target(), Map.of());
            case "admin.grant.create", "admin.grant.revoke" -> sendDaemon(source,
                command.spec().target(), grantBody(command));
            case "admin.principal.inspect" -> sendDaemon(source, "admin.principal.inspect",
                principalBody(command.argument("principal")));
            case "admin.audit.tail" -> sendDaemon(source, "admin.audit.tail",
                Map.of("lines", Integer.parseInt(command.argument("lines"))));
            case "adventure.purchase" -> sendDaemon(source, "adventure.purchase", Map.of(
                "adventureId", command.argument("adventure")));
            case "adventure.return" -> messageKey(source, "velocity.adventure.return", NamedTextColor.YELLOW);
            case "adventure.session.cancel" -> sendDaemon(source, "adventure.session.cancel", Map.of(
                "sessionId", command.argument("session"), "reason", command.argument("reason")));
            default -> message(source, "unsupported command", NamedTextColor.RED);
        }
    }
    private Map<String, Object> grantBody(CommandInvocation command) {
        var body = new HashMap<String, Object>(principalBody(command.argument("principal")));
        body.put("roleId", command.argument("role"));
        body.put("reason", command.argument("reason"));
        return body;
    }
    private Map<String, Object> principalBody(String principal) {
        var parts = principal.split(":", 2);
        return Map.of("subjectKind", parts.length == 2 ? parts[0] : "minecraft-player",
            "subjectId", parts.length == 2 ? parts[1] : principal);
    }
    private void status(CommandSource source) {
        message(source, "lkjmc velocity running; players=" + proxy.getPlayerCount(), NamedTextColor.GREEN);
        sendDaemon(source, "status", Map.of());
    }
    private void sendServerList(CommandSource source) {
        sendDaemon(source, "instance.list", Map.of());
    }
    private void reload(CommandSource source) {
        registry.ifPresent(VelocityServerRegistry::refresh);
        sendDaemon(source, "config.reload", Map.of());
    }
    private void doctor(CommandSource source) {
        var status = DaemonHttpConfigStatus.fromEnv();
        message(source, "lkjmc doctor: platform=velocity root=/lkjmc", NamedTextColor.GREEN);
        message(source, "daemon http: " + status.code(), status.configured() ? NamedTextColor.GREEN : NamedTextColor.RED);
        if (status.configured()) {
            sendDaemon(source, "doctor", Map.of());
        }
    }
    private void warnRestart(CommandSource source, String secondsText) {
        try {
            restart.scheduleWarning(Integer.parseInt(secondsText));
            message(source, "ok restart warning", NamedTextColor.GREEN);
        } catch (NumberFormatException error) {
            message(source, "usage: /lkjmc restart warn <seconds>", NamedTextColor.RED);
        }
    }
    private void sendDaemon(CommandSource source, String command, Map<String, Object> body) {
        if (daemon.isEmpty()) {
            message(source, "daemon unavailable: " + DaemonHttpConfigStatus.fromEnv().code(), NamedTextColor.RED);
            return;
        }
        var request = new DaemonRequest(UUID.randomUUID(), new DaemonActor("velocity-plugin", "velocity"),
            command, VelocityCommandPrincipal.body(source, command, body,
                (candidate, permission) -> candidate.hasPermission(permission)));
        daemon.get().send(request).thenAccept(response -> message(source,
            response.ok() ? format(command, response.body()) : response.error().map(error -> error.code()).orElse("failed"),
            response.ok() ? NamedTextColor.GREEN : NamedTextColor.RED));
    }
    private String format(String command, JsonObject body) {
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
    private void message(CommandSource source, String text, NamedTextColor color) {
        source.sendMessage(Component.text(text, color));
    }
    private void messageKey(CommandSource source, String key, NamedTextColor color) {
        source.sendMessage(VelocityMessages.message(key, color));
    }
}
