package com.lkjmc.velocity;

import static org.junit.jupiter.api.Assertions.assertEquals;

import com.lkjmc.common.permission.PermissionNodes;
import com.velocitypowered.api.command.CommandSource;
import com.velocitypowered.api.command.SimpleCommand;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.server.RegisteredServer;
import com.velocitypowered.api.proxy.server.ServerInfo;
import java.lang.reflect.Proxy;
import java.net.InetSocketAddress;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import org.junit.jupiter.api.Test;

final class VelocityLkjmcCommandTest {
    @Test
    void suggestsSharedTreeWithProxyContext() {
        var proxy = proxyServer(List.of(server("hub")), List.of(player("Alex")));
        var command = new VelocityLkjmcCommand(proxy, Optional.empty(), Optional.empty(),
            new VelocityRestartAdapter(proxy, new Object()), player -> java.util.concurrent.CompletableFuture.completedFuture(true));
        var source = source(Set.of(PermissionNodes.ADMIN_SEND, PermissionNodes.ADMIN_INSTANCE_START));
        assertEquals(List.of("send", "server"), command.suggest(invocation(source, "s")));
        assertEquals(List.of("Alex"), command.suggest(invocation(source, "send", "")));
        assertEquals(List.of("hub"), command.suggest(invocation(source, "send", "Alex", "")));
    }

    private static ProxyServer proxyServer(List<RegisteredServer> servers, List<Player> players) {
        return proxy(ProxyServer.class, (proxy, method, args) -> switch (method.getName()) {
            case "getAllServers" -> servers;
            case "getAllPlayers" -> players;
            default -> fallback(method.getReturnType());
        });
    }

    private static RegisteredServer server(String name) {
        var info = new ServerInfo(name, InetSocketAddress.createUnresolved("127.0.0.1", 25565));
        return proxy(RegisteredServer.class, (proxy, method, args) ->
            method.getName().equals("getServerInfo") ? info : fallback(method.getReturnType()));
    }

    private static Player player(String name) {
        return proxy(Player.class, (proxy, method, args) ->
            method.getName().equals("getUsername") ? name : fallback(method.getReturnType()));
    }

    private static CommandSource source(Set<String> permissions) {
        return proxy(CommandSource.class, (proxy, method, args) -> switch (method.getName()) {
            case "hasPermission" -> permissions.contains((String) args[0]);
            default -> fallback(method.getReturnType());
        });
    }

    private static SimpleCommand.Invocation invocation(CommandSource source, String... args) {
        return proxy(SimpleCommand.Invocation.class, (proxy, method, ignored) -> switch (method.getName()) {
            case "source" -> source;
            case "arguments" -> args;
            case "alias" -> "lkjmc";
            default -> fallback(method.getReturnType());
        });
    }

    @SuppressWarnings("unchecked")
    private static <T> T proxy(Class<T> type, java.lang.reflect.InvocationHandler handler) {
        return (T) Proxy.newProxyInstance(type.getClassLoader(), new Class<?>[] {type}, handler);
    }

    private static Object fallback(Class<?> type) {
        if (type.equals(boolean.class)) return false;
        if (type.equals(int.class)) return 0;
        if (type.equals(void.class)) return null;
        return null;
    }
}
