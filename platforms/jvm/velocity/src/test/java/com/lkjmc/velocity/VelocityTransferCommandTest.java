package com.lkjmc.velocity;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertSame;

import com.velocitypowered.api.command.CommandSource;
import com.velocitypowered.api.command.SimpleCommand;
import com.velocitypowered.api.proxy.ConnectionRequestBuilder;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.ServerConnection;
import com.velocitypowered.api.proxy.server.RegisteredServer;
import com.velocitypowered.api.proxy.server.ServerInfo;
import java.lang.reflect.Proxy;
import java.net.InetSocketAddress;
import java.util.ArrayList;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.junit.jupiter.api.Test;

final class VelocityTransferCommandTest {
    @Test
    void hubCommandSavesProfileBeforeConnecting() {
        var hub = server("hub");
        var player = player("Smoke", server("paper"));
        var proxy = proxyServer(Map.of("hub", hub), Map.of("Smoke", player.proxy));

        new VelocityHubCommand(proxy, saved()).execute(invocation(player.proxy));

        assertEquals(1, player.requests.size());
        assertSame(hub, player.requests.get(0));
    }

    @Test
    void sendCommandSavesProfileBeforeConnecting() {
        var hub = server("hub");
        var player = player("Smoke", server("paper"));
        var source = source();
        var proxy = proxyServer(Map.of("hub", hub), Map.of("Smoke", player.proxy));

        new VelocitySendAdapter(proxy, saved()).send(invocation(source.proxy), "Smoke", "hub");

        assertEquals(1, player.requests.size());
        assertSame(hub, player.requests.get(0));
        assertEquals(1, source.messages.size());
    }

    @Test
    void sendCommandDeniesSameServerTransfer() {
        var paper = server("paper");
        var player = player("Smoke", paper);
        var source = source();
        var proxy = proxyServer(Map.of("paper", paper), Map.of("Smoke", player.proxy));

        new VelocitySendAdapter(proxy, saved()).send(invocation(source.proxy), "Smoke", "paper");

        assertEquals(0, player.requests.size());
        assertEquals(1, source.messages.size());
    }

    private static ProfileSaveBridge saved() {
        return player -> CompletableFuture.completedFuture(true);
    }

    private static TestPlayer player(String name, RegisteredServer current) {
        var requests = new ArrayList<RegisteredServer>();
        var serverConnection = proxy(ServerConnection.class, (proxy, method, args) -> switch (method.getName()) {
            case "getServer" -> current;
            case "getServerInfo" -> current.getServerInfo();
            default -> fallback(method.getReturnType());
        });
        var player = proxy(Player.class, (proxy, method, args) -> switch (method.getName()) {
            case "getUsername" -> name;
            case "getUniqueId" -> UUID.nameUUIDFromBytes(name.getBytes());
            case "getCurrentServer" -> Optional.of(serverConnection);
            case "createConnectionRequest" -> request((RegisteredServer) args[0], requests);
            case "sendMessage" -> null;
            default -> fallback(method.getReturnType());
        });
        return new TestPlayer(player, requests);
    }

    private static ConnectionRequestBuilder request(RegisteredServer server, ArrayList<RegisteredServer> requests) {
        return proxy(ConnectionRequestBuilder.class, (proxy, method, args) -> {
            if (method.getName().equals("fireAndForget")) {
                requests.add(server);
                return null;
            }
            return method.getName().equals("getServer") ? server : fallback(method.getReturnType());
        });
    }

    private static RegisteredServer server(String name) {
        var info = new ServerInfo(name, InetSocketAddress.createUnresolved("127.0.0.1", 25565));
        return proxy(RegisteredServer.class, (proxy, method, args) ->
            method.getName().equals("getServerInfo") ? info : fallback(method.getReturnType()));
    }

    private static ProxyServer proxyServer(Map<String, RegisteredServer> servers, Map<String, Player> players) {
        return proxy(ProxyServer.class, (proxy, method, args) -> switch (method.getName()) {
            case "getServer" -> Optional.ofNullable(servers.get((String) args[0]));
            case "getPlayer" -> Optional.ofNullable(players.get((String) args[0]));
            default -> fallback(method.getReturnType());
        });
    }

    private static TestSource source() {
        var messages = new ArrayList<Object>();
        var source = proxy(CommandSource.class, (proxy, method, args) -> {
            if (method.getName().equals("sendMessage")) {
                messages.add(args[0]);
            }
            return fallback(method.getReturnType());
        });
        return new TestSource(source, messages);
    }

    private static SimpleCommand.Invocation invocation(CommandSource source) {
        return proxy(SimpleCommand.Invocation.class, (proxy, method, args) -> switch (method.getName()) {
            case "source" -> source;
            case "arguments" -> new String[0];
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

    private record TestPlayer(Player proxy, ArrayList<RegisteredServer> requests) {}
    private record TestSource(CommandSource proxy, ArrayList<Object> messages) {}
}
