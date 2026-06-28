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
import com.velocitypowered.api.command.SimpleCommand;
import com.velocitypowered.api.proxy.ProxyServer;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

final class VelocityLkjmcCommand implements SimpleCommand {
    private final ProxyServer proxy;
    private final Optional<DaemonClient> daemon;
    private final Optional<VelocityServerRegistry> registry;
    private final VelocityRestartAdapter restart;
    private final VelocitySendAdapter send;
    private final VelocityTemporarySendAdapter temporarySend;
    private final VelocityWakeJoinAdapter wakeJoin;

    VelocityLkjmcCommand(ProxyServer proxy, Optional<DaemonClient> daemon,
                         Optional<VelocityServerRegistry> registry,
                         VelocityRestartAdapter restart, ProfileSaveBridge transfers) {
        this.proxy = proxy;
        this.daemon = daemon == null ? Optional.empty() : daemon;
        this.registry = registry == null ? Optional.empty() : registry;
        this.restart = restart;
        this.send = new VelocitySendAdapter(proxy, transfers);
        this.temporarySend = new VelocityTemporarySendAdapter(proxy, this.daemon, send);
        this.wakeJoin = new VelocityWakeJoinAdapter(proxy, this.daemon, this.registry, send);
    }

    @Override
    public void execute(Invocation invocation) {
        var parsed = LkjmcCommandTree.parse(CommandPlatform.VELOCITY, List.of(invocation.arguments()));
        if (parsed.isEmpty()) {
            message(invocation, "usage: " + LkjmcCommandTree.usage(CommandPlatform.VELOCITY,
                List.of(invocation.arguments())), NamedTextColor.YELLOW);
            return;
        }
        execute(invocation, parsed.get());
    }

    @Override
    public boolean hasPermission(Invocation invocation) {
        return LkjmcCommandTree.parse(CommandPlatform.VELOCITY, List.of(invocation.arguments()))
            .map(value -> invocation.source().hasPermission(value.spec().permission()))
            .orElseGet(() -> LkjmcCommandTree.specs().stream()
                .filter(spec -> spec.supports(CommandPlatform.VELOCITY))
                .anyMatch(spec -> invocation.source().hasPermission(spec.permission())));
    }

    @Override
    public List<String> suggest(Invocation invocation) {
        return LkjmcCommandTree.suggest(CommandPlatform.VELOCITY, List.of(invocation.arguments()),
            invocation.source()::hasPermission, context());
    }

    private void execute(Invocation invocation, CommandInvocation command) {
        if (!invocation.source().hasPermission(command.spec().permission())) {
            message(invocation, "no permission: " + command.spec().permission(), NamedTextColor.RED);
            return;
        }
        switch (command.spec().target()) {
            case "status" -> message(invocation, "lkjmc velocity running; players=" + proxy.getPlayerCount(), NamedTextColor.GREEN);
            case "doctor" -> doctor(invocation);
            case "config.reload" -> reload(invocation);
            case "restart.warn" -> warnRestart(invocation, command.argument("seconds"));
            case "proxy.send" -> send.send(invocation, command.argument("player"), command.argument("server"));
            case "temporary.send" -> temporarySend.send(invocation, command.argument("player"), command.argument("instance"));
            case "wake.send" -> wakeJoin.send(invocation, command.argument("player"), command.argument("server"));
            case "instance.list" -> sendServerList(invocation);
            case "instance.create" -> sendDaemon(invocation, "instance.create", Map.of(
                "id", command.argument("server"), "kind", "paper", "template", command.argument("template")));
            case "instance.delete" -> sendDaemon(invocation, "instance.delete", Map.of(
                "id", command.argument("server"), "force", false));
            case "instance.start", "instance.stop", "instance.restart" -> sendDaemon(invocation,
                command.spec().target(), Map.of("id", command.argument("server")));
            default -> message(invocation, "unsupported command", NamedTextColor.RED);
        }
    }

    private CommandCompletionContext context() {
        var servers = proxy.getAllServers().stream().map(server -> server.getServerInfo().getName()).sorted().toList();
        var players = proxy.getAllPlayers().stream().map(player -> player.getUsername()).sorted().toList();
        return new CommandCompletionContext(servers, players, List.of("paper", "folia", "purpur"));
    }

    private void sendServerList(Invocation invocation) {
        var names = proxy.getAllServers().stream().map(server -> server.getServerInfo().getName()).sorted().toList();
        message(invocation, names.isEmpty() ? "servers: none" : "servers: " + String.join(", ", names), NamedTextColor.GREEN);
    }

    private void reload(Invocation invocation) {
        registry.ifPresent(VelocityServerRegistry::refresh);
        sendDaemon(invocation, "config.reload", Map.of());
    }

    private void doctor(Invocation invocation) {
        var status = DaemonHttpConfigStatus.fromEnv();
        message(invocation, "lkjmc doctor: platform=velocity root=/lkjmc", NamedTextColor.GREEN);
        message(invocation, "daemon http: " + status.code(), status.configured() ? NamedTextColor.GREEN : NamedTextColor.RED);
        if (status.configured()) {
            sendDaemon(invocation, "doctor", Map.of());
        }
    }

    private void warnRestart(Invocation invocation, String secondsText) {
        try {
            restart.scheduleWarning(Integer.parseInt(secondsText));
            message(invocation, "ok restart warning", NamedTextColor.GREEN);
        } catch (NumberFormatException error) {
            message(invocation, "usage: /lkjmc restart warn <seconds>", NamedTextColor.RED);
        }
    }

    private void sendDaemon(Invocation invocation, String command, Map<String, Object> body) {
        if (daemon.isEmpty()) {
            message(invocation, "daemon unavailable: " + DaemonHttpConfigStatus.fromEnv().code(), NamedTextColor.RED);
            return;
        }
        var request = new DaemonRequest(UUID.randomUUID(), new DaemonActor("velocity-plugin", "velocity"), command, body);
        daemon.get().send(request).thenAccept(response -> message(invocation,
            response.ok() ? format(command, response.body()) : response.error().map(error -> error.code()).orElse("failed"),
            response.ok() ? NamedTextColor.GREEN : NamedTextColor.RED));
    }

    private String format(String command, JsonObject body) {
        if (command.equals("instance.list") && body.has("instances") && body.get("instances").isJsonArray()) {
            return "ok instance.list";
        }
        return "ok " + command;
    }

    private void message(Invocation invocation, String text, NamedTextColor color) {
        invocation.source().sendMessage(Component.text(text, color));
    }
}
