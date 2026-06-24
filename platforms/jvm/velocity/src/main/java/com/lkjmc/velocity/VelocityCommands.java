package com.lkjmc.velocity;

import com.lkjmc.common.permission.PermissionNodes;
import com.velocitypowered.api.command.CommandMeta;
import com.velocitypowered.api.command.SimpleCommand;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import java.util.List;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityCommands {
    private final ProxyServer proxy;

    public VelocityCommands(ProxyServer proxy) {
        this.proxy = proxy;
    }

    public void register() {
        var commands = proxy.getCommandManager();
        CommandMeta lkjmc = commands.metaBuilder("lkjmc").build();
        commands.register(lkjmc, new LkjmcCommand(proxy));
        CommandMeta hub = commands.metaBuilder("hub").build();
        commands.register(hub, new HubCommand(proxy));
    }

    private static final class LkjmcCommand implements SimpleCommand {
        private final ProxyServer proxy;

        private LkjmcCommand(ProxyServer proxy) {
            this.proxy = proxy;
        }

        @Override
        public void execute(Invocation invocation) {
            var args = List.of(invocation.arguments());
            if (args.equals(List.of("status"))) {
                invocation.source().sendMessage(Component.text(
                    "lkjmc velocity running; players=" + proxy.getPlayerCount(),
                    NamedTextColor.GREEN
                ));
            } else if (args.equals(List.of("server", "list"))) {
                var names = proxy.getAllServers().stream()
                    .map(server -> server.getServerInfo().getName())
                    .sorted()
                    .toList();
                invocation.source().sendMessage(Component.text(
                    "servers: " + String.join(", ", names),
                    NamedTextColor.GREEN
                ));
            } else {
                invocation.source().sendMessage(Component.text(
                    "usage: /lkjmc status | /lkjmc server list",
                    NamedTextColor.YELLOW
                ));
            }
        }

        @Override
        public boolean hasPermission(Invocation invocation) {
            var args = List.of(invocation.arguments());
            if (args.equals(List.of("server", "list"))) {
                return invocation.source().hasPermission(PermissionNodes.ADMIN_INSTANCE_LIST);
            }
            return invocation.source().hasPermission(PermissionNodes.ADMIN_STATUS);
        }
    }

    private static final class HubCommand implements SimpleCommand {
        private final ProxyServer proxy;

        private HubCommand(ProxyServer proxy) {
            this.proxy = proxy;
        }

        @Override
        public void execute(Invocation invocation) {
            if (!(invocation.source() instanceof Player player)) {
                invocation.source().sendMessage(Component.text("players only", NamedTextColor.RED));
                return;
            }
            proxy.getServer("hub").ifPresentOrElse(
                server -> player.createConnectionRequest(server).fireAndForget(),
                () -> player.sendMessage(Component.text("hub unavailable", NamedTextColor.RED))
            );
        }
    }
}
