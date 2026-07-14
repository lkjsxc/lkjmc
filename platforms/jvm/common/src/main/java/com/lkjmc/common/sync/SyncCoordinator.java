package com.lkjmc.common.sync;
import com.google.gson.JsonObject;
import java.time.Duration;
import java.time.Instant;
import java.util.Optional;
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
    private final ConcurrentHashMap<SyncKey, CompletableFuture<Void>> snapshots = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<SyncKey, Long> required = new ConcurrentHashMap<>();
    private final AtomicBoolean feedFlight = new AtomicBoolean();
    private final AtomicBoolean closed = new AtomicBoolean();
    private final AtomicLong cursor;
    private volatile long credentialRevision;
    private volatile long nextPollNanos;
    private volatile int failures;
    public SyncCoordinator(SyncConfig config) {
        this(config, 0);
    }
    public SyncCoordinator(SyncConfig config, long initialCursor) {
        if (initialCursor < 0) {
            throw new IllegalArgumentException("sync cursor must not be negative");
        }
        this.config = config;
        this.http = new SyncHttpClient(config);
        this.cache = new SyncCache(config.maxEntries(), config.maxCacheBytes(), config.maxAge());
        this.cursor = new AtomicLong(initialCursor);
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
        return subscriptions.add(key) || subscriptions.contains(key);
    }
    public void unsubscribe(SyncKey key) {
        subscriptions.remove(key);
        CompletableFuture<Void> request = snapshots.remove(key);
        if (request != null) {
            request.cancel(true);
        }
    }
    public Optional<SyncSnapshot> view(SyncKey key) {
        return subscriptions.contains(key) ? cache.get(key, Instant.now()) : Optional.empty();
    }

    public long checkpoint() { return cursor.get(); }
    public void replaceCredential(String credential) {
        http.replaceCredential(credential);
        invalidate();
    }
    public int requestCount() { return http.inflight(); }
    public int subscriptionCount() { return subscriptions.size(); }
    private void tick() {
        if (closed.get()) {
            return;
        }
        Instant now = Instant.now();
        subscriptions.forEach(key -> {
            if (needsRefresh(key, now)) {
                refresh(key);
            }
        });
        if (System.nanoTime() < nextPollNanos || !feedFlight.compareAndSet(false, true)) {
            return;
        }
        JsonObject request = new JsonObject();
        request.addProperty("cursor", cursor.get());
        request.addProperty("limit", 128);
        http.post("/sync/feed", request).whenComplete((body, failure) -> {
            feedFlight.set(false);
            if (failure != null || !applyFeed(body)) {
                failed();
            } else {
                failures = 0;
                nextPollNanos = 0;
            }
        });
    }
    private boolean applyFeed(JsonObject body) {
        try {
            if (!acceptCredentialRevision(body.get("credentialRevision").getAsLong())) {
                return false;
            }
            String result = body.get("result").getAsString();
            long nextCursor = body.get("cursor").getAsLong();
            if ("reload-required".equals(result)) {
                cache.clear();
                cursor.set(nextCursor);
                return true;
            }
            if (!"changes".equals(result)) {
                return false;
            }
            body.getAsJsonArray("changes").forEach(element -> {
                JsonObject change = element.getAsJsonObject();
                SyncKey key = new SyncKey(change.get("domain").getAsString(), change.get("key").getAsString());
                if (subscriptions.contains(key)) {
                    required.merge(key, change.get("revision").getAsLong(), Math::max);
                    refresh(key);
                }
            });
            cursor.set(nextCursor);
            return true;
        } catch (RuntimeException invalid) {
            return false;
        }
    }
    private synchronized void refresh(SyncKey key) {
        if (closed.get() || snapshots.containsKey(key)) {
            return;
        }
        JsonObject request = new JsonObject();
        request.addProperty("domain", key.domain());
        request.addProperty("key", key.key());
        CompletableFuture<Void> result = http.post("/sync/snapshot", request)
                .thenAccept(body -> applySnapshot(key, body));
        snapshots.put(key, result);
        result.whenComplete((unused, failure) -> snapshots.remove(key, result));
    }
    private void applySnapshot(SyncKey expected, JsonObject body) {
        if (!"snapshot".equals(body.get("result").getAsString())) {
            throw new IllegalStateException("sync snapshot unavailable");
        }
        SyncKey actual = new SyncKey(body.get("domain").getAsString(), body.get("key").getAsString());
        if (!expected.equals(actual) || !acceptCredentialRevision(body.get("credentialRevision").getAsLong())) {
            throw new IllegalStateException("sync response invalidated");
        }
        int bytes = body.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8).length;
        SyncSnapshot value = new SyncSnapshot(actual, body.get("revision").getAsLong(),
                Instant.parse(body.get("generatedAt").getAsString()), body.get("payload"), bytes, Instant.now());
        cache.put(value, Instant.now());
        required.computeIfPresent(actual, (key, revision) -> value.revision() >= revision ? null : revision);
    }
    private boolean needsRefresh(SyncKey key, Instant now) {
        long wanted = required.getOrDefault(key, 0L);
        return cache.get(key, now).map(value -> value.revision() < wanted).orElse(true);
    }
    private synchronized boolean acceptCredentialRevision(long serverRevision) {
        if (credentialRevision != 0 && credentialRevision != serverRevision) {
            invalidate();
            return false;
        }
        credentialRevision = serverRevision;
        return true;
    }
    private void invalidate() {
        cache.clear();
        cursor.set(0);
        credentialRevision = 0;
        snapshots.values().forEach(future -> future.cancel(true));
        snapshots.clear();
        required.clear();
    }
    private void failed() {
        int count = Math.min(++failures, 17);
        nextPollNanos = System.nanoTime() + backoff.delay(count, FEED_KEY).toNanos();
    }

    @Override
    public void close() {
        if (closed.compareAndSet(false, true)) {
            scheduler.shutdownNow();
            snapshots.values().forEach(future -> future.cancel(true));
            snapshots.clear();
            http.close();
            cache.clear();
        }
    }

    public boolean awaitClosed(Duration timeout) throws InterruptedException {
        long deadline = System.nanoTime() + timeout.toNanos();
        if (!scheduler.awaitTermination(timeout.toMillis(), TimeUnit.MILLISECONDS)) {
            return false;
        }
        return http.awaitClosed(Duration.ofNanos(Math.max(0, deadline - System.nanoTime())));
    }
}
