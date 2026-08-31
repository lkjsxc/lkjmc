package com.lkjmc.velocity;

import static org.junit.jupiter.api.Assertions.*;

import com.google.gson.JsonParser;
import com.velocitypowered.api.command.CommandManager;
import com.velocitypowered.api.command.CommandMeta;
import com.velocitypowered.api.event.EventManager;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.server.RegisteredServer;
import com.velocitypowered.api.proxy.server.ServerInfo;
import java.lang.reflect.Proxy;
import java.net.InetSocketAddress;
import java.time.Duration;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.CompletionException;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;

final class VelocityLifecycleTest {
    @Test
    void oneHundredCyclesLeaveNoListenerRuntimeOrOverlappingThread() throws Exception {
        Set<Object> listeners = Collections.newSetFromMap(new IdentityHashMap<>());
        AtomicInteger maximumListeners = new AtomicInteger();
        EventManager events = (EventManager) Proxy.newProxyInstance(getClass().getClassLoader(),
                new Class<?>[] {EventManager.class}, (target, method, arguments) -> {
                    if (method.getName().equals("register") && arguments.length == 2) {
                        listeners.add(arguments[1]);
                        maximumListeners.accumulateAndGet(listeners.size(), Math::max);
                        return null;
                    }
                    if (method.getName().equals("unregisterListener")) {
                        listeners.remove(arguments[1]);
                        return null;
                    }
                    return defaultValue(method.getReturnType());
                });
        Set<Object> commands = Collections.newSetFromMap(new IdentityHashMap<>());
        AtomicInteger maximumCommands = new AtomicInteger();
        CommandManager commandManager = commandManager(commands, maximumCommands);
        ProxyServer proxy = proxy(events, commandManager, true);
        VelocityLifecycle lifecycle = new VelocityLifecycle(
                proxy, ignored -> {}, List.of("alpha-world", "beta-world"));
        Object plugin = new Object();
        for (int cycle = 0; cycle < 100; cycle++) lifecycle.initialize(plugin);
        lifecycle.close();
        assertTrue(lifecycle.awaitIdle(Duration.ofSeconds(10)));
        assertEquals(0, listeners.size());
        assertEquals(2, maximumListeners.get());
        assertEquals(0, commands.size());
        assertEquals(1, maximumCommands.get());
        assertEquals(0, lifecycle.activeRuntimes());
        assertEquals(1, lifecycle.maximumActiveRuntimes());
        assertTrue(Thread.getAllStackTraces().keySet().stream().noneMatch(thread -> thread.isAlive()
                && (thread.getName().startsWith("lkjmc-effect-velocity")
                    || thread.getName().equals("lkjmc-runtime-lifecycle"))));
    }

    @Test
    void missingConfiguredRouteFailsBeforeHeartbeatStarts() throws Exception {
        Set<Object> listeners = Collections.newSetFromMap(new IdentityHashMap<>());
        EventManager events = (EventManager) Proxy.newProxyInstance(getClass().getClassLoader(),
                new Class<?>[] {EventManager.class}, (target, method, arguments) -> {
                    if (method.getName().equals("register") && arguments.length == 2) {
                        listeners.add(arguments[1]);
                        return null;
                    }
                    if (method.getName().equals("unregisterListener")) {
                        listeners.remove(arguments[1]);
                        return null;
                    }
                    return defaultValue(method.getReturnType());
                });
        Set<Object> commands = Collections.newSetFromMap(new IdentityHashMap<>());
        List<String> diagnostics = new CopyOnWriteArrayList<>();
        VelocityLifecycle lifecycle = new VelocityLifecycle(
                proxy(events, commandManager(commands, new AtomicInteger()), false),
                diagnostics::add,
                List.of("alpha-world", "beta-world"));

        CompletionException failure = assertThrows(CompletionException.class,
                () -> lifecycle.initialize(new Object()).join());
        assertInstanceOf(IllegalStateException.class, failure.getCause());
        assertEquals("configured backend registration is missing: beta-world",
                failure.getCause().getMessage());
        assertTrue(diagnostics.stream().noneMatch(message ->
                message.startsWith("lkjmc Velocity backend registrations verified:")));
        assertTrue(diagnostics.contains("lkjmc Velocity initialization failed"));
        assertEquals(0, listeners.size());
        assertEquals(0, commands.size());
        assertEquals(0, lifecycle.activeRuntimes());

        lifecycle.close();
        assertTrue(lifecycle.awaitIdle(Duration.ofSeconds(10)));
    }

    @Test
    void heartbeatPayloadReportsTheLiveRegistrationResult() {
        RoutingPlatform platform = new VelocityProxyPlatform(proxy(null, null, true));
        List<VelocityLifecycle.ExpectedRegistration> expected = List.of(
                new VelocityLifecycle.ExpectedRegistration(
                        "alpha-world", new RoutingTarget("127.0.0.1", "alpha-world", 25566)),
                new VelocityLifecycle.ExpectedRegistration(
                        "beta-world", new RoutingTarget("127.0.0.1", "beta-world", 25568)));

        var registrations = JsonParser.parseString(
                        VelocityLifecycle.registrationPayload(platform, expected))
                .getAsJsonObject()
                .getAsJsonArray("registrations");
        assertEquals(2, registrations.size());
        assertTrue(registrations.get(0).getAsJsonObject().get("registered").getAsBoolean());
        assertFalse(registrations.get(1).getAsJsonObject().get("registered").getAsBoolean());
        assertEquals("route-mismatch", registrations.get(1).getAsJsonObject()
                .get("failureReason").getAsString());
        assertEquals(25568, registrations.get(1).getAsJsonObject()
                .get("connectPort").getAsInt());
    }

    private ProxyServer proxy(
            EventManager events, CommandManager commandManager, boolean includeSecondBackend) {
        return (ProxyServer) Proxy.newProxyInstance(getClass().getClassLoader(),
                new Class<?>[] {ProxyServer.class}, (target, method, arguments) -> switch (method.getName()) {
                    case "getEventManager" -> events;
                    case "getCommandManager" -> commandManager;
                    case "getServer" -> {
                        String id = (String) arguments[0];
                        int port = switch (id) {
                            case "alpha-world" -> 25566;
                            case "beta-world" -> includeSecondBackend ? 25567 : -1;
                            default -> -1;
                        };
                        yield port < 0 ? Optional.empty() : Optional.of(registeredServer(id, port));
                    }
                    default -> defaultValue(method.getReturnType());
                });
    }

    private RegisteredServer registeredServer(String id, int port) {
        ServerInfo info = new ServerInfo(id, new InetSocketAddress("127.0.0.1", port));
        return (RegisteredServer) Proxy.newProxyInstance(getClass().getClassLoader(),
                new Class<?>[] {RegisteredServer.class}, (target, method, arguments) ->
                        method.getName().equals("getServerInfo")
                                ? info : defaultValue(method.getReturnType()));
    }

    private CommandManager commandManager(Set<Object> commands, AtomicInteger maximumCommands) {
        return (CommandManager) Proxy.newProxyInstance(getClass().getClassLoader(),
                new Class<?>[] {CommandManager.class}, (target, method, arguments) -> {
                    if (method.getName().equals("metaBuilder")) {
                        return Proxy.newProxyInstance(getClass().getClassLoader(),
                                new Class<?>[] {CommandMeta.Builder.class}, (builder, operation, values) -> {
                                    if (operation.getName().equals("build")) {
                                        return Proxy.newProxyInstance(getClass().getClassLoader(),
                                                new Class<?>[] {CommandMeta.class},
                                                (meta, access, inputs) -> defaultValue(access.getReturnType()));
                                    }
                                    return builder;
                                });
                    }
                    if (method.getName().equals("register") && arguments.length == 2) {
                        commands.add(arguments[0]);
                        maximumCommands.accumulateAndGet(commands.size(), Math::max);
                        return null;
                    }
                    if (method.getName().equals("unregister") && arguments.length == 1) {
                        commands.remove(arguments[0]);
                        return null;
                    }
                    if (method.getName().equals("hasCommand")) {
                        return arguments.length == 1 && "lkjmc".equals(arguments[0])
                                && !commands.isEmpty();
                    }
                    return defaultValue(method.getReturnType());
                });
    }

    private static Object defaultValue(Class<?> type) {
        if (!type.isPrimitive()) return null;
        if (type == boolean.class) return false;
        if (type == char.class) return '\0';
        return 0;
    }
}
