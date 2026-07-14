package com.lkjmc.common.sync;

import com.google.gson.Gson;
import com.google.gson.JsonObject;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Semaphore;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;

final class SyncHttpClient implements AutoCloseable {
    private record Credential(long generation, String value) {}

    private final SyncConfig config;
    private final Gson gson = new Gson();
    private final ExecutorService executor;
    private final HttpClient client;
    private final Semaphore budget;
    private final Set<CompletableFuture<?>> inflight = ConcurrentHashMap.newKeySet();
    private final AtomicReference<Credential> credential;

    SyncHttpClient(SyncConfig config) {
        this.config = config;
        this.executor = Executors.newFixedThreadPool(Math.min(config.maxInflight(), 4), runnable -> {
            Thread thread = new Thread(runnable, "lkjmc-sync-http");
            thread.setDaemon(true);
            return thread;
        });
        this.client = HttpClient.newBuilder().connectTimeout(config.requestTimeout()).executor(executor).build();
        this.budget = new Semaphore(config.maxInflight());
        this.credential = new AtomicReference<>(new Credential(1, config.credential()));
    }

    long generation() {
        return credential.get().generation();
    }

    void replaceCredential(String value) {
        if (value == null || value.isBlank()) {
            throw new IllegalArgumentException("credential is required");
        }
        credential.updateAndGet(old -> new Credential(old.generation() + 1, value));
        inflight.forEach(future -> future.cancel(true));
    }

    CompletableFuture<JsonObject> post(String path, JsonObject body) {
        if (!budget.tryAcquire()) {
            return CompletableFuture.failedFuture(new IllegalStateException("sync request budget exhausted"));
        }
        Credential captured = credential.get();
        HttpRequest request = HttpRequest.newBuilder(config.endpoint().resolve(path))
                .timeout(config.requestTimeout())
                .header("Authorization", "Bearer " + captured.value())
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(gson.toJson(body), StandardCharsets.UTF_8))
                .build();
        CompletableFuture<JsonObject> result = client.sendAsync(request, HttpResponse.BodyHandlers.ofByteArray())
                .orTimeout(config.requestTimeout().toMillis(), TimeUnit.MILLISECONDS)
                .thenApply(response -> decode(response, captured.generation()));
        inflight.add(result);
        result.whenComplete((ignored, failure) -> {
            inflight.remove(result);
            budget.release();
        });
        return result;
    }

    int inflight() {
        return config.maxInflight() - budget.availablePermits();
    }

    private JsonObject decode(HttpResponse<byte[]> response, long generation) {
        if (generation != credential.get().generation()) {
            throw new IllegalStateException("credential generation changed");
        }
        byte[] body = response.body();
        if (response.statusCode() != 200 || body.length > config.maxResponseBytes()) {
            throw new IllegalStateException("sync response unavailable");
        }
        return gson.fromJson(new String(body, StandardCharsets.UTF_8), JsonObject.class);
    }

    @Override
    public void close() {
        inflight.forEach(future -> future.cancel(true));
        executor.shutdownNow();
        try {
            executor.awaitTermination(Duration.ofSeconds(2).toMillis(), TimeUnit.MILLISECONDS);
        } catch (InterruptedException exception) {
            Thread.currentThread().interrupt();
        }
    }
}
