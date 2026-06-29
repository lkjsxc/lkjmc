package com.lkjmc.velocity;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonResponse;
import com.lkjmc.common.permission.PermissionNodes;
import com.mojang.brigadier.CommandDispatcher;
import com.velocitypowered.api.command.BrigadierCommand;
import com.velocitypowered.api.command.CommandSource;
import com.velocitypowered.api.permission.Tristate;
import com.velocitypowered.api.proxy.ProxyServer;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.junit.jupiter.api.Test;

final class VelocityLkjmcCommandRegressionTest {
    @Test
    void deniedGraphPathStillReturnsProductNoPermissionCopy() throws Exception {
        var source = sourceWithMessages(Set.of());
        execute(command(Optional.empty()), source.proxy(), "lkjmc status");

        assertEquals(1, source.messages().size());
        assertTrue(source.messages().get(0).contains("no permission: " + PermissionNodes.ADMIN_STATUS));
    }

    @Test
    void serverListUsesDaemonInstancesInsteadOfProxyRegistry() throws Exception {
        var daemon = new FakeDaemon();
        var source = sourceWithMessages(Set.of(PermissionNodes.ADMIN_INSTANCE_LIST));
        execute(command(Optional.of(daemon)), source.proxy(), "lkjmc server list");

        assertEquals("instance.list", daemon.lastCommand);
        assertEquals(1, source.messages().size());
        assertTrue(source.messages().get(0).contains("servers: hub, smp"));
    }

    private static BrigadierCommand command(Optional<DaemonClient> daemon) {
        var proxy = proxyServer();
        var executor = new VelocityLkjmcCommand(proxy, daemon, Optional.empty(),
            new VelocityRestartAdapter(proxy, new Object()),
            player -> CompletableFuture.completedFuture(true));
        return VelocityLkjmcBrigadier.create(executor);
    }

    private static void execute(BrigadierCommand command, CommandSource source, String input) throws Exception {
        var dispatcher = new CommandDispatcher<CommandSource>();
        dispatcher.getRoot().addChild(command.getNode());
        dispatcher.execute(input, source);
    }

    private static ProxyServer proxyServer() {
        return proxy(ProxyServer.class, (proxy, method, args) -> switch (method.getName()) {
            case "getAllServers", "getAllPlayers" -> List.of();
            case "getPlayerCount" -> 0;
            default -> fallback(method.getReturnType());
        });
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

    private static final class FakeDaemon implements DaemonClient {
        private String lastCommand;

        @Override
        public CompletableFuture<DaemonResponse> send(com.lkjmc.common.daemon.DaemonRequest request) {
            lastCommand = request.command();
            var body = new JsonObject();
            var instances = new JsonArray();
            instances.add(instance("hub"));
            instances.add(instance("smp"));
            body.add("instances", instances);
            return CompletableFuture.completedFuture(new DaemonResponse(
                UUID.randomUUID(), true, body, Optional.empty()));
        }

        private static JsonObject instance(String id) {
            var object = new JsonObject();
            object.addProperty("id", id);
            return object;
        }
    }
}
