package com.lkjmc.common.sync;

import com.google.gson.JsonObject;
import java.time.Duration;
import java.time.Instant;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;

public final class SyncCoordinator implements AutoCloseable {
    private static final SyncKey FEED_KEY = new SyncKey("routing", "network");
    private final SyncConfig config;
    private final SyncHttpClient http;
    private final SyncCache cache;
    private final ReconnectBackoff backoff = new ReconnectBackoff(Duration.ofMillis(200), Duration.ofSeconds(10));
    private final ScheduledExecutorService scheduler;
    private final Set<SyncKey> subscriptions = ConcurrentHashMap.newKeySet();
    private final ConcurrentHashMap<SyncKey, CompletableFuture<Void>> singleFlight = new ConcurrentHashMap<>();
    private final AtomicBoolean feedFlight = new AtomicBoolean();
    private final AtomicBoolean closed = new AtomicBoolean();
    private final AtomicLong cursor = new AtomicLong();
    private volatile long credentialRevision;
    private volatile long nextPollNanos;
    private volatile int failures;

    public SyncCoordinator(SyncConfig config) {
        this.config = config;
        this.http = new SyncHttpClient(config);
        this.cache = new SyncCache(config.maxEntries(), config.maxCacheBytes(), config.maxAge());
        this.scheduler = Executors.newSingleThreadScheduledExecutor(runnable -> {
            Thread thread = new Thread(runnable, "lkjmc-sync-coordinator");
            thread.setDaemon(true);
            return thread;
        });
        scheduler.scheduleWithFixedDelay(this::tick, 0, config.pollInterval().toMillis(), TimeUnit.MILLISECONDS);
    }

    public boolean subscribe(SyncKey key) {
        if (closed.get() || (!subscriptions.contains(key) && subscriptions.size() >= config.maxSubscriptions())) {
            return false;
        }
        boolean added = subscriptions.add(key);
        if (added) {
            refresh(key);
        }
        return true;
    }

    public void unsubscribe(SyncKey key) {
        subscriptions.remove(key);
        CompletableFuture<Void> request = singleFlight.remove(key);
        if (request != null) {
            request.cancel(true);
        }
    }

    public java.util.Optional<SyncSnapshot> view(SyncKey key) {
        return subscriptions.contains(key) ? cache.get(key, Instant.now()) : java.util.Optional.empty();
    }

    public void replaceCredential(String credential) {
        http.replaceCredential(credential);
        invalidate();
    }

    public int requestCount() {
        return http.inflight();
    }

    public int subscriptionCount() {
        return subscriptions.size();
    }

    private void tick() {
        if (closed.get() || System.nanoTime() < nextPollNanos || !feedFlight.compareAndSet(false, true)) {
            return;
        }
        JsonObject request = new JsonObject();
        request.addProperty("cursor", cursor.get());
        request.addProperty("limit", 128);
        http.post("/sync/feed", request).whenComplete((body, failure) -> {
            feedFlight.set(false);
            if (failure != null) {
                failed();
                return;
            }
            try {
                applyFeed(body);
                failures = 0;
                nextPollNanos = 0;
            } catch (RuntimeException invalid) {
                failed();
            }
        });
    }

    private void applyFeed(JsonObject body) {
        long serverCredentialRevision = body.get("credentialRevision").getAsLong();
        if (credentialRevision != 0 && credentialRevision != serverCredentialRevision) {
            invalidate();
        }
        credentialRevision = serverCredentialRevision;
        String result = body.get("result").getAsString();
        cursor.set(body.get("cursor").getAsLong());
        if ("reload-required".equals(result)) {
            cache.clear();
            subscriptions.forEach(this::refresh);
            return;
        }
        if (!"changes".equals(result)) {
            throw new IllegalStateException("unexpected sync feed result");
        }
        body.getAsJsonArray("changes").forEach(element -> {
            JsonObject change = element.getAsJsonObject();
            SyncKey key = new SyncKey(change.get("domain").getAsString(), change.get("key").getAsString());
            if (subscriptions.contains(key)) {
                refresh(key);
            }
        });
    }

    private void refresh(SyncKey key) {
        CompletableFuture<Void> created;
        synchronized (singleFlight) {
            if (closed.get() || singleFlight.containsKey(key)) {
                return;
            }
            JsonObject request = new JsonObject();
            request.addProperty("domain", key.domain());
            request.addProperty("key", key.key());
            created = http.post("/sync/snapshot", request).thenAccept(body -> applySnapshot(key, body));
            singleFlight.put(key, created);
        }
        CompletableFuture<Void> tracked = created;
        created.whenComplete((ignored, failure) -> {
            singleFlight.remove(key, tracked);
            if (failure != null && !closed.get()) {
                failed();
            }
        });
    }

    private void applySnapshot(SyncKey expected, JsonObject body) {
        if (!"snapshot".equals(body.get("result").getAsString())) {
            throw new IllegalStateException("sync snapshot unavailable");
        }
        SyncKey actual = new SyncKey(body.get("domain").getAsString(), body.get("key").getAsString());
        if (!expected.equals(actual)) {
            throw new IllegalStateException("sync key mismatch");
        }
        long serverCredentialRevision = body.get("credentialRevision").getAsLong();
        if (credentialRevision != 0 && credentialRevision != serverCredentialRevision) {
            invalidate();
            throw new IllegalStateException("credential revision changed");
        }
        credentialRevision = serverCredentialRevision;
        int bytes = body.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8).length;
        SyncSnapshot snapshot = new SyncSnapshot(actual, body.get("revision").getAsLong(),
                Instant.parse(body.get("generatedAt").getAsString()), body.get("payload"), bytes, Instant.now());
        cache.put(snapshot, Instant.now());
    }

    private void invalidate() {
        cache.clear();
        cursor.set(0);
        credentialRevision = 0;
        singleFlight.values().forEach(future -> future.cancel(true));
        singleFlight.clear();
    }

    private void failed() {
        int count = Math.min(++failures, 17);
        nextPollNanos = System.nanoTime() + backoff.delay(count, FEED_KEY).toNanos();
    }

    @Override
    public void close() {
        if (!closed.compareAndSet(false, true)) {
            return;
        }
        scheduler.shutdownNow();
        singleFlight.values().forEach(future -> future.cancel(true));
        singleFlight.clear();
        http.close();
        cache.clear();
        try {
            scheduler.awaitTermination(2, TimeUnit.SECONDS);
        } catch (InterruptedException exception) {
            Thread.currentThread().interrupt();
        }
    }
}
