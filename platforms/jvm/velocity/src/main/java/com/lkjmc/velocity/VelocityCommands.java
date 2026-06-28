package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.permission.PermissionNodes;
import com.velocitypowered.api.command.CommandMeta;
import com.velocitypowered.api.command.SimpleCommand;
import com.velocitypowered.api.proxy.ProxyServer;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityCommands {
    private final ProxyServer proxy;
    private final Optional<DaemonClient> daemon;
    private final Optional<VelocityServerRegistry> registry;
    private final VelocityRestartAdapter restart;
    private final ProfileSaveBridge transfers;

    public VelocityCommands(
        ProxyServer proxy,
        Optional<DaemonClient> daemon,
        Optional<VelocityServerRegistry> registry,
        VelocityRestartAdapter restart,
        ProfileSaveBridge transfers
    ) {
        this.proxy = proxy;
        this.daemon = daemon == null ? Optional.empty() : daemon;
        this.registry = registry == null ? Optional.empty() : registry;
        this.restart = restart;
        this.transfers = transfers;
    }

    public void register() {
        var commands = proxy.getCommandManager();
        CommandMeta lkjmc = commands.metaBuilder("lkjmc").build();
        commands.register(lkjmc, new LkjmcCommand(proxy, daemon, registry, restart, transfers));
        CommandMeta hub = commands.metaBuilder("hub").build();
        commands.register(hub, new VelocityHubCommand(proxy, transfers));
    }

    private static final class LkjmcCommand implements SimpleCommand {
        private final ProxyServer proxy;
        private final Optional<DaemonClient> daemon;
        private final Optional<VelocityServerRegistry> registry;
        private final VelocityRestartAdapter restart;
        private final VelocitySendAdapter send;
        private final VelocityTemporarySendAdapter temporarySend;

        private LkjmcCommand(
            ProxyServer proxy,
            Optional<DaemonClient> daemon,
            Optional<VelocityServerRegistry> registry,
            VelocityRestartAdapter restart,
            ProfileSaveBridge transfers
        ) {
            this.proxy = proxy;
            this.daemon = daemon;
            this.registry = registry;
            this.restart = restart;
            this.send = new VelocitySendAdapter(proxy, transfers);
            this.temporarySend = new VelocityTemporarySendAdapter(proxy, daemon, send);
        }

        @Override
        public void execute(Invocation invocation) {
            var args = List.of(invocation.arguments());
            if (args.equals(List.of("status"))) {
                sendLocalStatus(invocation);
            } else if (args.equals(List.of("server", "list"))) {
                sendServerList(invocation);
            } else if (args.equals(List.of("reload"))) {
                reload(invocation);
            } else if (args.size() == 3 && args.equals(List.of("restart", "warn", args.get(2)))) {
                warnRestart(invocation, args.get(2));
            } else if (args.size() == 3 && args.get(0).equals("send")) {
                send.send(invocation, args.get(1), args.get(2));
            } else if (args.size() == 4 && args.get(0).equals("temporary") && args.get(1).equals("send")) {
                temporarySend.send(invocation, args.get(2), args.get(3));
            } else if (args.size() == 3 && args.get(0).equals("server")) {
                sendLifecycle(invocation, args.get(1), args.get(2));
            } else if (args.size() == 4 && args.equals(List.of("server", "delete", args.get(2), "confirm"))) {
                sendLifecycle(invocation, "delete", args.get(2));
            } else if (args.size() == 4 && args.get(0).equals("server") && args.get(1).equals("create")) {
                createServer(invocation, args.get(2), args.get(3));
            } else {
                invocation.source().sendMessage(Component.text(
                    "usage: /lkjmc status|reload|restart warn | /lkjmc server ...",
                    NamedTextColor.YELLOW
                ));
            }
        }

        @Override
        public boolean hasPermission(Invocation invocation) {
            var args = List.of(invocation.arguments());
            if (args.size() >= 2 && args.get(0).equals("server")) {
                return hasServerPermission(invocation, args.get(1));
            }
            if (!args.isEmpty() && (args.get(0).equals("send") || args.get(0).equals("temporary"))) {
                return invocation.source().hasPermission(PermissionNodes.ADMIN_SEND);
            }
            if (!args.isEmpty() && (args.get(0).equals("reload") || args.get(0).equals("restart"))) {
                return invocation.source().hasPermission(PermissionNodes.ADMIN_RELOAD);
            }
            return invocation.source().hasPermission(PermissionNodes.ADMIN_STATUS);
        }

        private void sendLocalStatus(Invocation invocation) {
            invocation.source().sendMessage(Component.text(
                "lkjmc velocity running; players=" + proxy.getPlayerCount(),
                NamedTextColor.GREEN
            ));
        }

        private void sendServerList(Invocation invocation) {
            var names = proxy.getAllServers().stream()
                .map(server -> server.getServerInfo().getName())
                .sorted()
                .toList();
            invocation.source().sendMessage(Component.text(
                "servers: " + String.join(", ", names),
                NamedTextColor.GREEN
            ));
        }

        private void reload(Invocation invocation) {
            registry.ifPresent(VelocityServerRegistry::refresh);
            invocation.source().sendMessage(Component.text("ok reload", NamedTextColor.GREEN));
        }

        private void warnRestart(Invocation invocation, String secondsText) {
            try {
                restart.scheduleWarning(Integer.parseInt(secondsText));
                invocation.source().sendMessage(Component.text("ok restart warning", NamedTextColor.GREEN));
            } catch (NumberFormatException error) {
                invocation.source().sendMessage(Component.text("invalid seconds", NamedTextColor.RED));
            }
        }

        private void sendLifecycle(Invocation invocation, String action, String id) {
            var command = switch (action) {
                case "start", "stop", "restart", "delete" -> "instance." + action;
                default -> "";
            };
            if (command.isBlank()) {
                invocation.source().sendMessage(Component.text("unknown server action", NamedTextColor.RED));
                return;
            }
            sendDaemon(invocation, command, Map.of("id", id, "force", false));
        }

        private void createServer(Invocation invocation, String id, String template) {
            sendDaemon(invocation, "instance.create", Map.of("id", id, "kind", "paper", "template", template));
        }

        private void sendDaemon(Invocation invocation, String command, Map<String, Object> body) {
            if (daemon.isEmpty()) {
                invocation.source().sendMessage(Component.text("daemon HTTP is not configured", NamedTextColor.RED));
                return;
            }
            var request = new DaemonRequest(
                UUID.randomUUID(),
                new DaemonActor("velocity-plugin", "velocity"),
                command,
                body
            );
            daemon.get().send(request).thenAccept(response -> invocation.source().sendMessage(
                Component.text(response.ok() ? "ok " + command : response.error().map(Object::toString).orElse("failed"),
                    response.ok() ? NamedTextColor.GREEN : NamedTextColor.RED)
            ));
        }

        private boolean hasServerPermission(Invocation invocation, String action) {
            return switch (action) {
                case "list" -> invocation.source().hasPermission(PermissionNodes.ADMIN_INSTANCE_LIST);
                case "create" -> invocation.source().hasPermission(PermissionNodes.ADMIN_INSTANCE_CREATE);
                case "start" -> invocation.source().hasPermission(PermissionNodes.ADMIN_INSTANCE_START);
                case "stop" -> invocation.source().hasPermission(PermissionNodes.ADMIN_INSTANCE_STOP);
                case "restart" -> invocation.source().hasPermission(PermissionNodes.ADMIN_INSTANCE_RESTART);
                case "delete" -> invocation.source().hasPermission(PermissionNodes.ADMIN_INSTANCE_DELETE);
                default -> invocation.source().hasPermission(PermissionNodes.ADMIN_STATUS);
            };
        }
    }
}
