package com.lkjmc.common.runtime;

import com.lkjmc.common.diagnostic.DiagnosticEmitter;
import com.lkjmc.common.effect.BoundedEffectExecutor;
import com.lkjmc.common.heartbeat.PluginHeartbeatReporter;
import com.lkjmc.common.sync.SyncCoordinator;
import com.lkjmc.common.sync.SyncKey;
import java.time.Duration;
import java.util.Collection;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Consumer;

public final class JvmPluginRuntime implements AutoCloseable {
    private final Optional<SyncCoordinator> coordinator;
    private final BoundedEffectExecutor effects;
    private final DiagnosticEmitter diagnostics;
    private final Optional<PluginHeartbeatReporter> heartbeat;
    private final AtomicBoolean closed = new AtomicBoolean();

    public JvmPluginRuntime(Optional<SyncCoordinator> coordinator, String owner) {
        this(coordinator, owner, System.err::println);
    }

    public JvmPluginRuntime(Optional<SyncCoordinator> coordinator, String owner, Consumer<String> sink) {
        this.coordinator = coordinator;
        this.effects = new BoundedEffectExecutor(owner, 2, 128);
        this.diagnostics = new DiagnosticEmitter(owner, sink);
        this.heartbeat = PluginHeartbeatReporter.fromEnvironment(System.getenv(), sink);
    }

    public void subscribe(Collection<SyncKey> keys) {
        if (closed.get()) throw new IllegalStateException("runtime closed");
        coordinator.ifPresent(value -> keys.forEach(value::subscribe));
    }

    public Optional<SyncCoordinator> coordinator() {
        return coordinator;
    }

    public BoundedEffectExecutor effects() {
        return effects;
    }

    public DiagnosticEmitter diagnostics() { return diagnostics; }

    public void startHeartbeat() {
        if (closed.get()) throw new IllegalStateException("runtime closed");
        heartbeat.ifPresent(PluginHeartbeatReporter::start);
    }

    @Override
    public void close() {
        if (closed.compareAndSet(false, true)) {
            heartbeat.ifPresent(PluginHeartbeatReporter::close);
            effects.close();
            coordinator.ifPresent(SyncCoordinator::close);
            diagnostics.close(Duration.ofSeconds(2));
        }
    }

    public CompletableFuture<Boolean> closeAsync(Duration timeout) {
        close();
        return CompletableFuture.supplyAsync(() -> {
            try {
                return awaitClosed(timeout);
            } catch (InterruptedException interrupted) {
                Thread.currentThread().interrupt();
                return false;
            }
        }, command -> Thread.ofVirtual().name("lkjmc-shutdown-join").start(command));
    }

    public boolean awaitClosed(Duration timeout) throws InterruptedException {
        long deadline = System.nanoTime() + timeout.toNanos();
        if (heartbeat.isPresent() && !heartbeat.get().awaitClosed(timeout)) return false;
        Duration left = Duration.ofNanos(Math.max(0, deadline - System.nanoTime()));
        if (!effects.awaitClosed(left)) return false;
        Duration coordinatorWait = Duration.ofNanos(Math.max(0, deadline - System.nanoTime()));
        return coordinator.map(value -> await(value, coordinatorWait)).orElse(true);
    }

    private boolean await(SyncCoordinator value, Duration timeout) {
        try {
            return value.awaitClosed(timeout);
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            return false;
        }
    }
}
