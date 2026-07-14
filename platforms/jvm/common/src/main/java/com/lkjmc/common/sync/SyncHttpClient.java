package com.lkjmc.common.sync;

import com.google.gson.Gson;
import com.google.gson.JsonObject;
import java.io.IOException;
import java.io.InputStream;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.Set;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.CancellationException;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Semaphore;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;

final class SyncHttpClient implements AutoCloseable {
    private record Credential(long generation, String value) {}

    private final SyncConfig config;
    private final Gson gson = new Gson();
    private final ThreadPoolExecutor executor;
    private final HttpClient client;
    private final Semaphore budget;
    private final Set<CompletableFuture<?>> inflight = java.util.concurrent.ConcurrentHashMap.newKeySet();
    private final Set<CompletableFuture<?>> wires = java.util.concurrent.ConcurrentHashMap.newKeySet();
    private final AtomicReference<Credential> credential;
    private final AtomicBoolean closed = new AtomicBoolean();

    SyncHttpClient(SyncConfig config) {
        this.config = config;
        int workers = Math.min(config.maxInflight(), 4);
        executor = new ThreadPoolExecutor(workers, workers, 0, TimeUnit.MILLISECONDS,
                new ArrayBlockingQueue<>(4096), runnable -> {
                    Thread thread = new Thread(runnable, "lkjmc-sync-http");
                    thread.setDaemon(true);
                    return thread;
                }, new ThreadPoolExecutor.AbortPolicy());
        client = HttpClient.newBuilder().connectTimeout(config.requestTimeout()).executor(executor).build();
        budget = new Semaphore(config.maxInflight());
        credential = new AtomicReference<>(new Credential(1, config.credential()));
    }

    void replaceCredential(String value) {
        if (value == null || value.isBlank()) throw new IllegalArgumentException("credential is required");
        credential.updateAndGet(old -> new Credential(old.generation() + 1, value));
        cancelRequests("sync credential replaced");
    }

    synchronized CompletableFuture<JsonObject> post(String path, JsonObject body) {
        if (closed.get() || !budget.tryAcquire()) {
            return CompletableFuture.failedFuture(new IllegalStateException("sync request unavailable"));
        }
        Credential captured = credential.get();
        CompletableFuture<JsonObject> tracked = new CompletableFuture<>();
        inflight.add(tracked);
        CompletableFuture<HttpResponse<InputStream>> wire;
        try {
            HttpRequest request = HttpRequest.newBuilder(config.endpoint().resolve(path))
                    .timeout(config.requestTimeout())
                    .header("Authorization", "Bearer " + captured.value())
                    .header("Content-Type", "application/json")
                    .POST(HttpRequest.BodyPublishers.ofString(gson.toJson(body), StandardCharsets.UTF_8))
                    .build();
            wire = client.sendAsync(request, HttpResponse.BodyHandlers.ofInputStream())
                    .orTimeout(config.requestTimeout().toMillis(), TimeUnit.MILLISECONDS);
            wires.add(wire);
        } catch (RuntimeException failure) {
            tracked.completeExceptionally(new IllegalStateException("sync request unavailable"));
            finish(tracked);
            return tracked;
        }
        tracked.whenComplete((unused, failure) -> {
            if (tracked.isCancelled() || closed.get()) wire.cancel(true);
            finish(tracked);
        });
        wire.whenComplete((response, failure) -> {
            wires.remove(wire);
            if (failure != null) {
                tracked.completeExceptionally(new IllegalStateException("sync request unavailable"));
            } else {
                try {
                    tracked.complete(decode(response, captured.generation()));
                } catch (RuntimeException invalid) {
                    tracked.completeExceptionally(invalid);
                }
            }
            finish(tracked);
        });
        return tracked;
    }

    int inflight() { return config.maxInflight() - budget.availablePermits(); }

    private JsonObject decode(HttpResponse<InputStream> response, long generation) {
        try (InputStream input = response.body()) {
            byte[] body = input.readNBytes(config.maxResponseBytes() + 1);
            if (generation != credential.get().generation() || response.statusCode() != 200
                    || body.length > config.maxResponseBytes()) {
                throw new IllegalStateException("sync response unavailable");
            }
            JsonObject decoded = gson.fromJson(new String(body, StandardCharsets.UTF_8), JsonObject.class);
            if (decoded == null) throw new IllegalStateException("sync response unavailable");
            return decoded;
        } catch (IOException failure) {
            throw new IllegalStateException("sync response unavailable");
        }
    }

    private void finish(CompletableFuture<?> request) {
        if (inflight.remove(request)) budget.release();
    }

    private void cancelRequests(String reason) {
        wires.forEach(future -> future.cancel(true));
        inflight.forEach(future -> future.completeExceptionally(new CancellationException(reason)));
    }

    @Override
    public synchronized void close() {
        if (!closed.compareAndSet(false, true)) return;
        cancelRequests("sync client closed");
        client.shutdownNow();
        executor.shutdownNow();
        cancelRequests("sync client closed");
    }

    boolean awaitClosed(Duration timeout) throws InterruptedException {
        long deadline = System.nanoTime() + timeout.toNanos();
        if (!client.awaitTermination(remaining(deadline))) return false;
        long left = Math.max(0, deadline - System.nanoTime());
        return executor.awaitTermination(left, TimeUnit.NANOSECONDS)
                && inflight.isEmpty() && wires.isEmpty() && client.isTerminated();
    }

    private static Duration remaining(long deadline) {
        return Duration.ofNanos(Math.max(0, deadline - System.nanoTime()));
    }
}
