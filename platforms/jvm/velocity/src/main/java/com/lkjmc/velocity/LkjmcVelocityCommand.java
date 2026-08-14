package com.lkjmc.velocity;

import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.velocitypowered.api.command.BrigadierCommand;
import com.velocitypowered.api.command.CommandSource;
import com.velocitypowered.api.proxy.ConnectionRequestBuilder;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.server.RegisteredServer;
import com.velocitypowered.api.proxy.server.ServerPing;
import java.time.Duration;
import java.util.List;
import java.util.Locale;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Semaphore;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Consumer;
import net.kyori.adventure.text.Component;

/** The deliberately small player-facing command surface owned by Velocity. */
public final class LkjmcVelocityCommand implements AutoCloseable {
    static final List<String> SERVER_IDS = List.of("hub", "survival");
    private static final Duration STATUS_TIMEOUT = Duration.ofSeconds(3);
    private static final Duration TRANSFER_TIMEOUT = Duration.ofSeconds(5);

    private final ProxyServer proxy;
    private final Consumer<String> diagnosticSink;
    private final long statusTimeoutMillis;
    private final long transferTimeoutMillis;
    private final Semaphore statusSlots = new Semaphore(8);
    private final Semaphore transferSlots = new Semaphore(32);
    private final AtomicBoolean closed = new AtomicBoolean();
    private final BrigadierCommand command;

    public LkjmcVelocityCommand(ProxyServer proxy, Consumer<String> diagnosticSink) {
        this(proxy, diagnosticSink, STATUS_TIMEOUT, TRANSFER_TIMEOUT);
    }

    LkjmcVelocityCommand(
            ProxyServer proxy,
            Consumer<String> diagnosticSink,
            Duration statusTimeout,
            Duration transferTimeout) {
        if (statusTimeout == null || statusTimeout.isZero() || statusTimeout.isNegative()
                || transferTimeout == null || transferTimeout.isZero() || transferTimeout.isNegative()) {
            throw new IllegalArgumentException("positive command timeouts required");
        }
        this.proxy = proxy;
        this.diagnosticSink = diagnosticSink;
        this.statusTimeoutMillis = Math.max(1, statusTimeout.toMillis());
        this.transferTimeoutMillis = Math.max(1, transferTimeout.toMillis());
        this.command = new BrigadierCommand(node());
    }

    public BrigadierCommand command() {
        return command;
    }

    private LiteralArgumentBuilder<CommandSource> node() {
        return BrigadierCommand.literalArgumentBuilder("lkjmc")
                .executes(context -> help(context.getSource()))
                .then(BrigadierCommand.literalArgumentBuilder("status")
                        .executes(context -> status(context.getSource())))
                .then(BrigadierCommand.literalArgumentBuilder("server")
                        .executes(context -> serverHelp(context.getSource()))
                        .then(BrigadierCommand.requiredArgumentBuilder(
                                        "id", StringArgumentType.word())
                                .suggests((context, suggestions) -> {
                                    String prefix = suggestions.getRemainingLowerCase();
                                    SERVER_IDS.stream().filter(id -> id.startsWith(prefix))
                                            .forEach(suggestions::suggest);
                                    return suggestions.buildFuture();
                                })
                                .executes(context -> transfer(
                                        context.getSource(),
                                        StringArgumentType.getString(context, "id")))));
    }

    private int help(CommandSource source) {
        source.sendMessage(Component.text("lkjmc: /lkjmc status | /lkjmc server <hub|survival>"));
        return 1;
    }

    private int serverHelp(CommandSource source) {
        source.sendMessage(Component.text("Usage: /lkjmc server <hub|survival>"));
        return 1;
    }

    private int status(CommandSource source) {
        if (closed.get()) {
            source.sendMessage(Component.text("lkjmc status is unavailable while the proxy is stopping."));
            return 0;
        }
        String current = source instanceof Player player
                ? player.getCurrentServer()
                        .map(server -> server.getServerInfo().getName())
                        .orElse("connecting")
                : "console";
        if (!statusSlots.tryAcquire()) {
            source.sendMessage(Component.text("lkjmc status is busy; try again shortly."));
            return 0;
        }

        List<Probe> probes = SERVER_IDS.stream().map(this::probe).toList();
        CompletableFuture.allOf(probes.stream().map(Probe::settled)
                        .toArray(CompletableFuture[]::new))
                .whenComplete((ignored, failure) -> statusSlots.release());
        CompletableFuture.allOf(probes.stream().map(Probe::response)
                        .toArray(CompletableFuture[]::new))
                .whenComplete((ignored, failure) -> {
                    if (closed.get()) return;
                    source.sendMessage(Component.text("lkjmc network: current=" + current));
                    probes.forEach(probe ->
                            source.sendMessage(Component.text(probe.response().join())));
                });
        return 1;
    }

    private Probe probe(String serverId) {
        try {
            RegisteredServer server = proxy.getServer(serverId).orElse(null);
            if (server == null) {
                return completedProbe(serverId + ": unavailable (not registered)");
            }
            CompletableFuture<ServerPing> operation = server.ping();
            if (operation == null) {
                return completedProbe(serverId + ": unavailable (ping failed)");
            }
            CompletableFuture<String> response = new CompletableFuture<>();
            CompletableFuture<Void> settled = new CompletableFuture<>();
            response.completeOnTimeout(
                    serverId + ": unavailable (ping timed out)",
                    statusTimeoutMillis,
                    TimeUnit.MILLISECONDS);
            operation.whenComplete((pingValue, failure) -> {
                try {
                    if (failure != null || pingValue == null) {
                        response.complete(serverId + ": unavailable (ping failed)");
                    } else {
                        response.complete(pingValue.getPlayers()
                                .map(players -> serverId + ": online players="
                                        + players.getOnline() + "/" + players.getMax())
                                .orElse(serverId + ": online players=unknown"));
                    }
                } catch (RuntimeException malformed) {
                    diagnostic("status probe returned invalid data for " + serverId, malformed);
                    response.complete(serverId + ": unavailable (invalid ping response)");
                } finally {
                    settled.complete(null);
                }
            });
            return new Probe(response, settled);
        } catch (RuntimeException failure) {
            diagnostic("status probe failed to start for " + serverId, failure);
            return completedProbe(serverId + ": unavailable (ping failed)");
        }
    }

    private Probe completedProbe(String response) {
        return new Probe(
                CompletableFuture.completedFuture(response),
                CompletableFuture.completedFuture(null));
    }

    private int transfer(CommandSource source, String requestedId) {
        String serverId = requestedId.toLowerCase(Locale.ROOT);
        if (!SERVER_IDS.contains(serverId)) {
            source.sendMessage(Component.text("Unknown server '" + requestedId
                    + "'. Choose hub or survival."));
            return 0;
        }
        if (!(source instanceof Player player)) {
            source.sendMessage(Component.text("Only a connected player can change servers."));
            return 0;
        }
        if (closed.get()) {
            player.sendMessage(Component.text("Server transfer is unavailable while the proxy is stopping."));
            return 0;
        }
        RegisteredServer server = proxy.getServer(serverId).orElse(null);
        if (server == null) {
            player.sendMessage(Component.text("Server '" + serverId + "' is not registered."));
            return 0;
        }
        if (!transferSlots.tryAcquire()) {
            player.sendMessage(Component.text("Server transfer is busy; try again shortly."));
            return 0;
        }

        player.sendMessage(Component.text("Connecting to " + serverId + "..."));
        try {
            CompletableFuture<ConnectionRequestBuilder.Result> operation =
                    player.createConnectionRequest(server).connect();
            if (operation == null) throw new IllegalStateException("connection future missing");
            CompletableFuture<TransferCompletion> feedback = new CompletableFuture<>();
            feedback.completeOnTimeout(
                    new TransferCompletion(null, null, true),
                    transferTimeoutMillis,
                    TimeUnit.MILLISECONDS);
            feedback.thenAccept(completion -> completeTransfer(player, serverId, completion));
            operation.whenComplete((result, failure) -> {
                transferSlots.release();
                feedback.complete(new TransferCompletion(result, failure, false));
            });
        } catch (RuntimeException failure) {
            transferSlots.release();
            diagnostic("transfer failed to start for " + serverId, failure);
            player.sendMessage(Component.text("Could not start transfer to " + serverId + "."));
            return 0;
        }
        return 1;
    }

    private void completeTransfer(Player player, String serverId, TransferCompletion completion) {
        if (closed.get()) return;
        Throwable failure = rootCause(completion.failure());
        if (completion.timedOut() || failure instanceof TimeoutException) {
            diagnostic("transfer timed out for " + serverId, failure);
            player.sendMessage(Component.text("Transfer to " + serverId + " timed out."));
            return;
        }
        ConnectionRequestBuilder.Result result = completion.result();
        if (failure != null || result == null || result.getStatus() == null) {
            diagnostic("transfer failed for " + serverId, failure);
            player.sendMessage(Component.text("Transfer to " + serverId + " failed."));
            return;
        }
        switch (result.getStatus()) {
            case SUCCESS -> player.sendMessage(Component.text("Connected to " + serverId + "."));
            case ALREADY_CONNECTED ->
                    player.sendMessage(Component.text("You are already connected to " + serverId + "."));
            case CONNECTION_IN_PROGRESS ->
                    player.sendMessage(Component.text("A server transfer is already in progress."));
            case CONNECTION_CANCELLED ->
                    player.sendMessage(Component.text("Transfer to " + serverId + " was cancelled."));
            case SERVER_DISCONNECTED ->
                    player.sendMessage(Component.text("Transfer to " + serverId
                            + " failed because the server disconnected."));
        }
    }

    private Throwable rootCause(Throwable failure) {
        Throwable current = failure;
        while (current != null && current.getCause() != null && current.getCause() != current) {
            current = current.getCause();
        }
        return current;
    }

    private void diagnostic(String message, Throwable failure) {
        String suffix = failure == null ? "" : ": " + failure.getClass().getSimpleName();
        try {
            diagnosticSink.accept("lkjmc velocity command: " + message + suffix);
        } catch (RuntimeException ignored) {
            // Diagnostics must never replace player feedback or leak an admission permit.
        }
    }

    @Override
    public void close() {
        closed.set(true);
    }

    private record Probe(CompletableFuture<String> response, CompletableFuture<Void> settled) {}

    private record TransferCompletion(
            ConnectionRequestBuilder.Result result, Throwable failure, boolean timedOut) {}
}
