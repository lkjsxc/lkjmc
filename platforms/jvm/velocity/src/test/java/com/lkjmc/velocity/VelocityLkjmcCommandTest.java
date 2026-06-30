package com.lkjmc.velocity;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.permission.PermissionNodes;
import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.suggestion.Suggestion;
import com.velocitypowered.api.command.BrigadierCommand;
import com.velocitypowered.api.command.CommandSource;
import com.velocitypowered.api.permission.Tristate;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.server.RegisteredServer;
import com.velocitypowered.api.proxy.server.ServerInfo;
import java.lang.reflect.Proxy;
import java.net.InetSocketAddress;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import org.junit.jupiter.api.Test;

final class VelocityLkjmcCommandTest {
    @Test
    void brigadierGraphSuggestsSharedTreeWithProxyContext() throws Exception {
        var command = command(proxyServer(List.of(server("hub")), List.of(player("Alex"))));
        var source = source(PermissionNodes.all());
        var graph = VelocityLkjmcBrigadier.create(command);

        assertEquals(List.of("security", "send", "server", "status"), suggestions(graph, source, "lkjmc s"));
        assertEquals(List.of("create", "delete", "list", "restart", "start", "stop"),
            suggestions(graph, source, "lkjmc server "));
        assertEquals(List.of("Alex"), suggestions(graph, source, "lkjmc send "));
        assertEquals(List.of("hub"), suggestions(graph, source, "lkjmc send Alex "));
    }

    @Test
    void brigadierNodesCarryPermissionRequirements() {
        var graph = VelocityLkjmcBrigadier.create(command(proxyServer(List.of(), List.of())));
        var limited = source(Set.of(PermissionNodes.ADMIN_INSTANCE_LIST));
        var server = graph.getNode().getChild("server");

        assertTrue(server.getChild("list").canUse(limited));
        assertFalse(server.getChild("create").canUse(limited));
    }

    @Test
    void brigadierGraphExecutesIntermediateUsage() throws Exception {
        var source = sourceWithMessages(Set.of(PermissionNodes.ADMIN_INSTANCE_LIST));
        execute(VelocityLkjmcBrigadier.create(command(proxyServer(List.of(), List.of()))), source.proxy(), "lkjmc server");

        assertEquals(1, source.messages().size());
        assertTrue(source.messages().get(0).contains("/lkjmc server list|start|stop|restart|create|delete"));
        assertFalse(source.messages().get(0).contains("position"));
    }

    @Test
    void brigadierGraphExecutesStatusWithProductOutput() throws Exception {
        var source = sourceWithMessages(Set.of(PermissionNodes.ADMIN_STATUS));
        execute(VelocityLkjmcBrigadier.create(command(proxyServer(List.of(), List.of()))), source.proxy(), "lkjmc status");

        assertEquals(2, source.messages().size());
        assertTrue(source.messages().get(0).contains("lkjmc velocity running"));
        assertTrue(source.messages().get(1).contains("daemon unavailable"));
    }

    @Test
    void executorKeepsSharedSuggestionsForSimpleCallers() {
        var proxy = proxyServer(List.of(server("hub")), List.of(player("Alex")));
        var command = command(proxy);
        var source = source(Set.of(PermissionNodes.ADMIN_SEND, PermissionNodes.ADMIN_INSTANCE_START));
        assertEquals(List.of("send", "server"), command.suggest(source, List.of("s")));
        assertTrue(command.hasAnyPermission(source));
    }

    private static VelocityLkjmcCommand command(ProxyServer proxy) {
        return new VelocityLkjmcCommand(proxy, Optional.empty(), Optional.empty(),
            new VelocityRestartAdapter(proxy, new Object()), player -> java.util.concurrent.CompletableFuture.completedFuture(true));
    }

    private static List<String> suggestions(BrigadierCommand command, CommandSource source, String input) throws Exception {
        var dispatcher = dispatcher(command);
        var parse = dispatcher.parse(input, source);
        return dispatcher.getCompletionSuggestions(parse).get().getList().stream().map(Suggestion::getText).toList();
    }

    private static void execute(BrigadierCommand command, CommandSource source, String input) throws Exception {
        dispatcher(command).execute(input, source);
    }

    private static CommandDispatcher<CommandSource> dispatcher(BrigadierCommand command) {
        var dispatcher = new CommandDispatcher<CommandSource>();
        dispatcher.getRoot().addChild(command.getNode());
        return dispatcher;
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
        return sourceWithMessages(permissions).proxy();
    }

    private static TestSource sourceWithMessages(Set<String> permissions) {
        var messages = new ArrayList<String>();
        var source = proxy(CommandSource.class, (proxy, method, args) -> switch (method.getName()) {
            case "hasPermission" -> permissions.contains((String) args[0]);
            case "getPermissionValue" -> Tristate.fromBoolean(permissions.contains((String) args[0]));
            case "sendMessage" -> {
                messages.add(String.valueOf(args[0]));
                yield null;
            }
            default -> fallback(method.getReturnType());
        });
        return new TestSource(source, messages);
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

    private record TestSource(CommandSource proxy, ArrayList<String> messages) {}
}
