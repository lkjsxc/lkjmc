package com.lkjmc.common.effect;

import java.time.Duration;
import java.util.Set;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.CancellationException;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.Future;
import java.util.concurrent.FutureTask;
import java.util.concurrent.ThreadFactory;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;

public final class BoundedEffectExecutor implements AutoCloseable {
    private final ThreadPoolExecutor workers;
    private final Set<Future<?>> active = java.util.concurrent.ConcurrentHashMap.newKeySet();
    private final Set<CompletableFuture<?>> results = java.util.concurrent.ConcurrentHashMap.newKeySet();
    private final AtomicBoolean closed = new AtomicBoolean();

    public BoundedEffectExecutor(String owner, int threads, int queueCapacity) {
        if (owner == null || owner.isBlank() || threads < 1 || threads > 16
                || queueCapacity < 1 || queueCapacity > 4096) {
            throw new IllegalArgumentException("invalid executor bounds");
        }
        AtomicInteger number = new AtomicInteger();
        ThreadFactory factory = task -> {
            Thread thread = new Thread(task, "lkjmc-effect-" + owner + "-" + number.incrementAndGet());
            thread.setDaemon(true);
            return thread;
        };
        workers = new ThreadPoolExecutor(threads, threads, 0, TimeUnit.MILLISECONDS,
                new ArrayBlockingQueue<>(queueCapacity), factory, new ThreadPoolExecutor.AbortPolicy());
    }

    public <T> CompletableFuture<T> submit(EffectTask<T> task) {
        CompletableFuture<T> result = new CompletableFuture<>();
        if (closed.get()) return CompletableFuture.failedFuture(new CancellationException("executor closed"));
        results.add(result);
        FutureTask<Void> submitted = new FutureTask<>(() -> {
            execute(task, result);
            return null;
        });
        active.add(submitted);
        result.whenComplete((unused, failure) -> {
            active.remove(submitted);
            results.remove(result);
            if (result.isCancelled()) submitted.cancel(true);
        });
        try {
            workers.execute(submitted);
        } catch (java.util.concurrent.RejectedExecutionException overloaded) {
            active.remove(submitted);
            results.remove(result);
            if (closed.get()) result.completeExceptionally(new CancellationException("executor closed"));
            else result.completeExceptionally(new IllegalStateException("effect queue overloaded"));
        }
        return result;
    }

    private <T> void execute(EffectTask<T> task, CompletableFuture<T> result) {
        Throwable last = new IllegalStateException("effect unavailable");
        for (int attempt = 1; attempt <= task.maxAttempts() && !closed.get(); attempt++) {
            CompletableFuture<T> stage = null;
            try {
                stage = task.operation().get().toCompletableFuture();
                CompletableFuture<T> captured = stage;
                result.whenComplete((unused, failure) -> {
                    if (result.isCancelled()) captured.cancel(true);
                });
                T value = stage.get(task.timeout().toMillis(), TimeUnit.MILLISECONDS);
                result.complete(value);
                return;
            } catch (TimeoutException timeout) {
                last = timeout;
            } catch (ExecutionException failed) {
                last = failed.getCause() == null ? failed : failed.getCause();
            } catch (InterruptedException cancelled) {
                Thread.currentThread().interrupt();
                result.completeExceptionally(new CancellationException("effect interrupted"));
                return;
            } catch (RuntimeException failed) {
                last = failed;
            } finally {
                if (stage != null && !stage.isDone()) stage.cancel(true);
            }
        }
        if (closed.get()) result.completeExceptionally(new CancellationException("executor closed"));
        else result.completeExceptionally(last);
    }

    public int activeCount() {
        return active.size();
    }

    public int queuedCount() {
        return workers.getQueue().size();
    }

    public void cancelAll() {
        active.forEach(future -> future.cancel(true));
        active.clear();
        results.forEach(result -> result.completeExceptionally(new CancellationException("executor closed")));
        results.clear();
        workers.getQueue().clear();
    }

    @Override
    public void close() {
        if (closed.compareAndSet(false, true)) {
            cancelAll();
            workers.shutdownNow();
        }
    }

    public boolean awaitClosed(Duration timeout) throws InterruptedException {
        return workers.awaitTermination(timeout.toMillis(), TimeUnit.MILLISECONDS) && active.isEmpty();
    }
}
