package com.lkjmc.velocity;

import static org.junit.jupiter.api.Assertions.*;

import com.mojang.brigadier.CommandDispatcher;
import com.velocitypowered.api.command.CommandSource;
import com.velocitypowered.api.proxy.ConnectionRequestBuilder;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.server.RegisteredServer;
import com.velocitypowered.api.proxy.server.ServerInfo;
import com.velocitypowered.api.proxy.server.ServerPing;
import java.lang.reflect.Proxy;
import java.net.InetSocketAddress;
import java.time.Duration;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.BooleanSupplier;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.TextComponent;
import org.junit.jupiter.api.Test;

final class LkjmcVelocityCommandTest {
    @Test
    void brigadierOffersOnlyTheSupportedSurface() throws Exception {
        Fixture fixture = new Fixture(ConnectionRequestBuilder.Status.SUCCESS);
        CommandSource source = fixture.source(false);

        assertEquals(List.of("server", "status"), fixture.suggestions("lkjmc ", source));
        assertEquals(List.of("copper-field", "quartz-world"), fixture.suggestions("lkjmc server ", source));
        assertEquals(List.of("quartz-world"), fixture.suggestions("lkjmc server q", source));
    }

    @Test
    void noncanonicalBackendInventoryDrivesCompletionAndValidation() throws Exception {
        Fixture fixture = new Fixture(
                ConnectionRequestBuilder.Status.SUCCESS,
                List.of("alpha-world", "beta-world", "gamma-world"));
        CommandSource source = fixture.source(true);

        assertEquals(
                List.of("alpha-world", "beta-world", "gamma-world"),
                fixture.suggestions("lkjmc server ", source));
        assertEquals(1, fixture.execute("lkjmc server gamma-world", source));
        assertEquals("gamma-world", fixture.requestedServer.get());
        fixture.messages.clear();
        assertEquals(0, fixture.execute("lkjmc server hub", source));
        assertEquals(
                "Unknown server 'hub'. Available instances: alpha-world, beta-world, gamma-world.",
                fixture.messages().getFirst());
    }

    @Test
    void statusQueriesBothRealRegisteredServerBoundaries() throws Exception {
        Fixture fixture = new Fixture(ConnectionRequestBuilder.Status.SUCCESS);
        CommandSource source = fixture.source(false);

        assertEquals(1, fixture.execute("lkjmc status", source));
        assertEquals(List.of(
                "lkjmc network: current=console",
                "copper-field: online players=2/20",
                "quartz-world: online players=2/20"), fixture.messages());
        assertEquals(List.of("copper-field", "quartz-world"), fixture.pingedServers);
    }

    @Test
    void successfulAndFailedTransfersReturnTruthfulPlayerFeedback() throws Exception {
        Fixture success = new Fixture(ConnectionRequestBuilder.Status.SUCCESS);
        CommandSource player = success.source(true);
        assertEquals(1, success.execute("lkjmc server quartz-world", player));
        assertEquals("quartz-world", success.requestedServer.get());
        assertEquals(
                List.of("Connecting to quartz-world...", "Connected to quartz-world."),
                success.messages());

        Fixture failure = new Fixture(ConnectionRequestBuilder.Status.SERVER_DISCONNECTED);
        CommandSource failedPlayer = failure.source(true);
        assertEquals(1, failure.execute("lkjmc server copper-field", failedPlayer));
        assertEquals(List.of(
                "Connecting to copper-field...",
                "Transfer to copper-field failed because the server disconnected."),
                failure.messages());
    }

    @Test
    void transferRejectsConsoleAndUnknownTargetWithoutRequestingConnection() throws Exception {
        Fixture fixture = new Fixture(ConnectionRequestBuilder.Status.SUCCESS);
        assertEquals(0, fixture.execute("lkjmc server copper-field", fixture.source(false)));
        assertEquals("Only a connected player can change servers.", fixture.messages().getFirst());
        assertNull(fixture.requestedServer.get());

        fixture.messages.clear();
        assertEquals(0, fixture.execute("lkjmc server creative", fixture.source(true)));
        assertEquals(
                "Unknown server 'creative'. Available instances: copper-field, quartz-world.",
                fixture.messages().getFirst());
        assertNull(fixture.requestedServer.get());
    }

    @Test
    void timedOutStatusWorkRetainsItsAdmissionSlotUntilThePingActuallySettles() throws Exception {
        Fixture fixture = new Fixture(
                ConnectionRequestBuilder.Status.SUCCESS,
                Duration.ofMillis(25),
                Duration.ofSeconds(5));
        fixture.holdPings = true;
        CommandSource source = fixture.source(false);
        for (int request = 0; request < 8; request++) {
            assertEquals(1, fixture.execute("lkjmc status", source));
        }
        await(() -> fixture.messages().stream()
                .filter(message -> message.contains("ping timed out"))
                .count() == 16);

        assertEquals(0, fixture.execute("lkjmc status", source));
        assertEquals("lkjmc status is busy; try again shortly.", fixture.messages().getLast());

        fixture.holdPings = false;
        fixture.pingOperations.forEach(operation -> operation.complete(fixture.ping("settled")));
        assertEquals(1, fixture.execute("lkjmc status", source));
    }

    @Test
    void timedOutTransfersRetainTheirSlotsAndExceptionalFailureIsNotCalledATimeout()
            throws Exception {
        Fixture fixture = new Fixture(
                ConnectionRequestBuilder.Status.SUCCESS,
                Duration.ofSeconds(3),
                Duration.ofMillis(25));
        fixture.holdTransfers = true;
        CommandSource player = fixture.source(true);
        for (int request = 0; request < 32; request++) {
            assertEquals(1, fixture.execute("lkjmc server copper-field", player));
        }
        await(() -> fixture.messages().stream()
                .filter(message -> message.equals("Transfer to copper-field timed out."))
                .count() == 32);

        assertEquals(0, fixture.execute("lkjmc server copper-field", player));
        assertEquals("Server transfer is busy; try again shortly.", fixture.messages().getLast());

        fixture.holdTransfers = false;
        fixture.transferOperations.forEach(operation ->
                operation.complete(fixture.connectionResult(fixture.servers.get("copper-field"))));
        assertEquals(1, fixture.execute("lkjmc server copper-field", player));

        Fixture failed = new Fixture(ConnectionRequestBuilder.Status.SUCCESS);
        failed.transferFailure = new IllegalStateException("synthetic connection failure");
        assertEquals(1, failed.execute("lkjmc server quartz-world", failed.source(true)));
        assertEquals("Transfer to quartz-world failed.", failed.messages().getLast());
    }

    @Test
    void closeImmediatelyRejectsNewWorkAndSuppressesLateFeedback() throws Exception {
        Fixture fixture = new Fixture(ConnectionRequestBuilder.Status.SUCCESS);
        fixture.holdPings = true;
        fixture.holdTransfers = true;
        CommandSource player = fixture.source(true);
        assertEquals(1, fixture.execute("lkjmc status", player));
        assertEquals(1, fixture.execute("lkjmc server quartz-world", player));
        assertEquals(List.of("Connecting to quartz-world..."), fixture.messages());

        fixture.command.close();
        fixture.pingOperations.forEach(operation -> operation.complete(fixture.ping("late")));
        fixture.transferOperations.forEach(operation ->
                operation.complete(fixture.connectionResult(fixture.servers.get("quartz-world"))));
        assertEquals(List.of("Connecting to quartz-world..."), fixture.messages());
        assertEquals(0, fixture.execute("lkjmc status", player));
        assertEquals(
                "lkjmc status is unavailable while the proxy is stopping.",
                fixture.messages().getLast());
    }

    @Test
    void everyVelocityTransferStatusHasSpecificFeedback() throws Exception {
        Map<ConnectionRequestBuilder.Status, String> expected = Map.of(
                ConnectionRequestBuilder.Status.ALREADY_CONNECTED,
                "You are already connected to copper-field.",
                ConnectionRequestBuilder.Status.CONNECTION_IN_PROGRESS,
                "A server transfer is already in progress.",
                ConnectionRequestBuilder.Status.CONNECTION_CANCELLED,
                "Transfer to copper-field was cancelled.");
        for (var entry : expected.entrySet()) {
            Fixture fixture = new Fixture(entry.getKey());
            assertEquals(1, fixture.execute("lkjmc server copper-field", fixture.source(true)));
            assertEquals(entry.getValue(), fixture.messages().getLast());
        }
    }

    private final class Fixture {
        private final List<Component> messages = new CopyOnWriteArrayList<>();
        private final List<String> pingedServers = new CopyOnWriteArrayList<>();
        private final List<CompletableFuture<ServerPing>> pingOperations =
                new CopyOnWriteArrayList<>();
        private final List<CompletableFuture<ConnectionRequestBuilder.Result>> transferOperations =
                new CopyOnWriteArrayList<>();
        private final AtomicReference<String> requestedServer = new AtomicReference<>();
        private final Map<String, RegisteredServer> servers = new LinkedHashMap<>();
        private final ConnectionRequestBuilder.Status transferStatus;
        private boolean holdPings;
        private boolean holdTransfers;
        private RuntimeException transferFailure;
        private final ProxyServer proxy;
        private final LkjmcVelocityCommand command;
        private final CommandDispatcher<CommandSource> dispatcher = new CommandDispatcher<>();

        private Fixture(ConnectionRequestBuilder.Status transferStatus) {
            this(transferStatus, Duration.ofSeconds(3), Duration.ofSeconds(5));
        }

        private Fixture(
                ConnectionRequestBuilder.Status transferStatus, List<String> serverIds) {
            this(transferStatus, Duration.ofSeconds(3), Duration.ofSeconds(5), serverIds);
        }

        private Fixture(
                ConnectionRequestBuilder.Status transferStatus,
                Duration statusTimeout,
                Duration transferTimeout) {
            this(transferStatus, statusTimeout, transferTimeout, List.of("copper-field", "quartz-world"));
        }

        private Fixture(
                ConnectionRequestBuilder.Status transferStatus,
                Duration statusTimeout,
                Duration transferTimeout,
                List<String> serverIds) {
            this.transferStatus = transferStatus;
            int port = 25566;
            for (String serverId : serverIds) {
                servers.put(serverId, server(serverId, port++));
            }
            proxy = (ProxyServer) Proxy.newProxyInstance(getClass().getClassLoader(),
                    new Class<?>[] {ProxyServer.class}, (target, method, arguments) -> {
                        if (method.getName().equals("getServer")) {
                            return Optional.ofNullable(servers.get(arguments[0]));
                        }
                        if (method.getName().equals("getAllServers")) {
                            return Set.copyOf(servers.values());
                        }
                        return defaultValue(method.getReturnType());
            });
            command = new LkjmcVelocityCommand(
                    proxy, ignored -> {}, serverIds, statusTimeout, transferTimeout);
            dispatcher.getRoot().addChild(command.command().getNode());
        }

        private RegisteredServer server(String id, int port) {
            ServerInfo info = new ServerInfo(id, new InetSocketAddress("127.0.0.1", port));
            return (RegisteredServer) Proxy.newProxyInstance(getClass().getClassLoader(),
                    new Class<?>[] {RegisteredServer.class}, (target, method, arguments) -> {
                        if (method.getName().equals("getServerInfo")) return info;
                        if (method.getName().equals("ping")) {
                            pingedServers.add(id);
                            CompletableFuture<ServerPing> operation = new CompletableFuture<>();
                            pingOperations.add(operation);
                            if (!holdPings) operation.complete(ping(id));
                            return operation;
                        }
                        return defaultValue(method.getReturnType());
                    });
        }

        private CommandSource source(boolean player) {
            Class<?>[] interfaces = player
                    ? new Class<?>[] {Player.class}
                    : new Class<?>[] {CommandSource.class};
            return (CommandSource) Proxy.newProxyInstance(getClass().getClassLoader(), interfaces,
                    (target, method, arguments) -> {
                        if (method.getName().equals("sendMessage")) {
                            for (Object argument : arguments) {
                                if (argument instanceof Component component) messages.add(component);
                            }
                            return null;
                        }
                        if (method.getName().equals("getCurrentServer")) return Optional.empty();
                        if (method.getName().equals("createConnectionRequest")) {
                            RegisteredServer destination = (RegisteredServer) arguments[0];
                            requestedServer.set(destination.getServerInfo().getName());
                            return connectionRequest(destination);
                        }
                        return defaultValue(method.getReturnType());
                    });
        }

        private ConnectionRequestBuilder connectionRequest(RegisteredServer destination) {
            return (ConnectionRequestBuilder) Proxy.newProxyInstance(getClass().getClassLoader(),
                    new Class<?>[] {ConnectionRequestBuilder.class}, (target, method, arguments) -> {
                        if (method.getName().equals("getServer")) return destination;
                        if (method.getName().equals("connect")) {
                            CompletableFuture<ConnectionRequestBuilder.Result> operation =
                                    new CompletableFuture<>();
                            transferOperations.add(operation);
                            if (transferFailure != null) {
                                operation.completeExceptionally(transferFailure);
                            } else if (!holdTransfers) {
                                operation.complete(connectionResult(destination));
                            }
                            return operation;
                        }
                        return defaultValue(method.getReturnType());
                    });
        }

        private ServerPing ping(String id) {
            return ServerPing.builder()
                    .version(new ServerPing.Version(774, "Folia"))
                    .onlinePlayers(2)
                    .maximumPlayers(20)
                    .description(Component.text(id))
                    .build();
        }

        private ConnectionRequestBuilder.Result connectionResult(RegisteredServer destination) {
            return (ConnectionRequestBuilder.Result) Proxy.newProxyInstance(
                    getClass().getClassLoader(),
                    new Class<?>[] {ConnectionRequestBuilder.Result.class},
                    (target, method, arguments) -> switch (method.getName()) {
                        case "getStatus" -> transferStatus;
                        case "getAttemptedConnection" -> destination;
                        case "getReasonComponent" -> Optional.empty();
                        case "isSuccessful" -> transferStatus == ConnectionRequestBuilder.Status.SUCCESS;
                        default -> defaultValue(method.getReturnType());
                    });
        }

        private int execute(String input, CommandSource source) throws Exception {
            return dispatcher.execute(input, source);
        }

        private List<String> suggestions(String input, CommandSource source) throws Exception {
            return dispatcher.getCompletionSuggestions(dispatcher.parse(input, source)).get().getList().stream()
                    .map(suggestion -> suggestion.getText())
                    .sorted()
                    .toList();
        }

        private List<String> messages() {
            return messages.stream().map(component -> ((TextComponent) component).content()).toList();
        }
    }

    private static void await(BooleanSupplier condition) throws InterruptedException {
        long deadline = System.nanoTime() + Duration.ofSeconds(2).toNanos();
        while (!condition.getAsBoolean() && System.nanoTime() < deadline) {
            Thread.sleep(5);
        }
        assertTrue(condition.getAsBoolean(), "condition did not become true before deadline");
    }

    private static Object defaultValue(Class<?> type) {
        if (type == boolean.class) return false;
        if (type == char.class) return '\0';
        if (type == byte.class || type == short.class || type == int.class || type == long.class) return 0;
        if (type == float.class || type == double.class) return 0.0;
        return null;
    }
}
