package com.lkjmc.common.runtime;

import java.time.Duration;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.function.Consumer;
import java.util.function.Supplier;

public final class SerializedRuntimeOwner {
    private final Object lock = new Object();
    private final Duration timeout;
    private CompletableFuture<Void> tail = CompletableFuture.completedFuture(null);
    private JvmPluginRuntime runtime;
    private int active;
    private int maximumActive;

    public SerializedRuntimeOwner(Duration timeout) {
        if (timeout == null || timeout.isNegative() || timeout.isZero()) {
            throw new IllegalArgumentException("positive shutdown timeout required");
        }
        this.timeout = timeout;
    }

    public CompletableFuture<Void> replace(
            Runnable uninstall,
            Supplier<JvmPluginRuntime> factory,
            Consumer<JvmPluginRuntime> install) {
        return enqueue(() -> {
            uninstall.run();
            shutdownCurrent();
            JvmPluginRuntime candidate = factory.get();
            runtime = candidate;
            synchronized (this) {
                active++;
                maximumActive = Math.max(maximumActive, active);
            }
            try {
                install.accept(candidate);
            } catch (RuntimeException failure) {
                shutdownCurrent();
                throw failure;
            }
        });
    }

    public CompletableFuture<Void> closeAsync(Runnable uninstall) {
        return enqueue(() -> {
            uninstall.run();
            shutdownCurrent();
        });
    }

    public boolean awaitIdle(Duration wait) throws InterruptedException {
        CompletableFuture<Void> current;
        synchronized (lock) {
            current = tail;
        }
        try {
            current.get(wait.toMillis(), TimeUnit.MILLISECONDS);
            return true;
        } catch (java.util.concurrent.TimeoutException | java.util.concurrent.ExecutionException failure) {
            return false;
        }
    }

    public synchronized int activeRuntimes() {
        return active;
    }

    public synchronized int maximumActiveRuntimes() {
        return maximumActive;
    }

    private CompletableFuture<Void> enqueue(Runnable action) {
        synchronized (lock) {
            tail = tail.handle((unused, failure) -> null).thenRunAsync(action,
                    command -> Thread.ofVirtual().name("lkjmc-runtime-lifecycle").start(command));
            return tail;
        }
    }

    private void shutdownCurrent() {
        JvmPluginRuntime current = runtime;
        if (current == null) return;
        runtime = null;
        current.close();
        try {
            if (!current.awaitClosed(timeout)) {
                throw new IllegalStateException("runtime shutdown timed out");
            }
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            throw new IllegalStateException("runtime shutdown interrupted", interrupted);
        } finally {
            synchronized (this) {
                active--;
            }
        }
    }
}
