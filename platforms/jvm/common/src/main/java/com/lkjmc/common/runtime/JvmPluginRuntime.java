package com.lkjmc.common.runtime;

import com.lkjmc.common.effect.BoundedEffectExecutor;
import com.lkjmc.common.sync.SyncCoordinator;
import com.lkjmc.common.sync.SyncKey;
import java.time.Duration;
import java.util.Collection;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicBoolean;

public final class JvmPluginRuntime implements AutoCloseable {
    private final Optional<SyncCoordinator> coordinator;
    private final BoundedEffectExecutor effects;
    private final AtomicBoolean closed = new AtomicBoolean();

    public JvmPluginRuntime(Optional<SyncCoordinator> coordinator, String owner) {
        this.coordinator = coordinator;
        this.effects = new BoundedEffectExecutor(owner, 2, 128);
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

    @Override
    public void close() {
        if (closed.compareAndSet(false, true)) {
            effects.close();
            coordinator.ifPresent(SyncCoordinator::close);
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
        if (!effects.awaitClosed(timeout)) return false;
        long left = Math.max(0, deadline - System.nanoTime());
        return coordinator.map(value -> await(value, Duration.ofNanos(left))).orElse(true);
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
