package com.lkjmc.common.sync;
import com.google.gson.JsonObject;
import com.lkjmc.bindings.FeedResponse;
import com.lkjmc.bindings.ReloadRequired;
import com.lkjmc.bindings.TypedSnapshot;
import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;
import java.util.function.LongSupplier;
public final class SyncCoordinator implements AutoCloseable {
    private static final SyncKey FEED_KEY = new SyncKey("routing", "network");
    private final SyncConfig config;
    private final SyncHttpClient http;
    private final SyncCache cache;
    private final ReconnectBackoff backoff = new ReconnectBackoff(Duration.ofMillis(200), Duration.ofSeconds(2));
    private final LongSupplier clock;
    private final ClosedSyncDecoder decoder = new ClosedSyncDecoder();
    private final RetryGate feedRetry;
    private final ScheduledExecutorService scheduler;
    private final Set<SyncKey> subscriptions = ConcurrentHashMap.newKeySet();
    private final ConcurrentHashMap<SyncKey, CompletableFuture<Void>> snapshots = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<SyncKey, RetryGate> snapshotRetries = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<SyncKey, Long> required = new ConcurrentHashMap<>();
    private final AtomicBoolean feedFlight = new AtomicBoolean();
    private final AtomicBoolean closed = new AtomicBoolean();
    private final AtomicLong cursor;
    private volatile long credentialRevision;
    public SyncCoordinator(SyncConfig config) { this(config, 0); }
    public SyncCoordinator(SyncConfig config, long initialCursor) {
        this(config, initialCursor, System::nanoTime);
    }
    SyncCoordinator(SyncConfig config, long initialCursor, LongSupplier clock) {
        if (initialCursor < 0) throw new IllegalArgumentException("sync cursor must not be negative");
        this.config = config;
        this.clock = clock;
        this.feedRetry = new RetryGate(backoff, FEED_KEY, clock);
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
        if (closed.get() || (!subscriptions.contains(key) && subscriptions.size() >= config.maxSubscriptions())) return false;
        return subscriptions.add(key) || subscriptions.contains(key);
    }
    public void unsubscribe(SyncKey key) {
        subscriptions.remove(key);
        snapshotRetries.remove(key);
        CompletableFuture<Void> request = snapshots.remove(key);
        if (request != null) request.cancel(true);
    }
    public Optional<SyncSnapshot> view(SyncKey key) {
        return subscriptions.contains(key) ? cache.get(key, Instant.now()) : Optional.empty();
    }
    public long checkpoint() { return cursor.get(); }
    public void replaceCredential(String credential) { http.replaceCredential(credential); invalidate(); }
    public int requestCount() { return http.inflight(); }
    public int subscriptionCount() { return subscriptions.size(); }
    private void tick() {
        if (closed.get()) return;
        Instant now = Instant.now();
        subscriptions.forEach(key -> {
            RetryGate retry = snapshotRetries.computeIfAbsent(key, item -> new RetryGate(backoff, item, clock));
            if (needsRefresh(key, now) && retry.canAttempt()) refresh(key, retry);
        });
        if (!feedRetry.canAttempt() || !feedFlight.compareAndSet(false, true)) return;
        JsonObject request = new JsonObject();
        request.addProperty("cursor", cursor.get());
        request.addProperty("limit", 128);
        http.post("/sync/feed", request).thenAccept(this::applyFeed).whenComplete((unused, failure) -> {
            feedFlight.set(false);
            if (failure == null) feedRetry.succeeded(); else feedRetry.failed();
        });
    }
    void applyFeed(JsonObject body) {
        var decoded = decoder.decode(body);
        boolean reload = decoded instanceof ReloadRequired;
        require(reload || decoded instanceof FeedResponse);
        long nextCursor = reload ? ((ReloadRequired) decoded).cursor() : ((FeedResponse) decoded).cursor();
        long serverRevision = reload ? ((ReloadRequired) decoded).credentialRevision()
                : ((FeedResponse) decoded).credentialRevision();
        require(nextCursor >= cursor.get() && acceptCredentialRevision(serverRevision));
        if (reload) {
            cache.clear();
            required.clear();
        } else {
            ((FeedResponse) decoded).changes().forEach(item -> {
                SyncKey key = new SyncKey(item.domain(), item.key());
                if (subscriptions.contains(key)) required.merge(key, item.revision(), Math::max);
            });
        }
        cursor.set(nextCursor);
    }
    private synchronized void refresh(SyncKey key, RetryGate retry) {
        if (closed.get() || snapshots.containsKey(key) || !retry.canAttempt()) return;
        JsonObject request = new JsonObject();
        request.addProperty("domain", key.domain()); request.addProperty("key", key.key());
        CompletableFuture<Void> result = http.post("/sync/snapshot", request).thenAccept(body -> applySnapshot(key, body));
        snapshots.put(key, result);
        result.whenComplete((unused, failure) -> {
            snapshots.remove(key, result);
            if (failure == null) retry.succeeded(); else retry.failed();
        });
    }
    void applySnapshot(SyncKey expected, JsonObject body) {
        var decoded = decoder.decode(body);
        require(decoded instanceof TypedSnapshot);
        TypedSnapshot snapshot = (TypedSnapshot) decoded;
        SyncKey actual = new SyncKey(snapshot.domain(), snapshot.key());
        require(expected.equals(actual) && acceptCredentialRevision(snapshot.credentialRevision()));
        int bytes = body.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8).length;
        SyncSnapshot value = new SyncSnapshot(snapshot, bytes, Instant.now());
        cache.put(value, Instant.now());
        cache.get(actual, Instant.now()).ifPresent(current -> required.computeIfPresent(actual,
                (key, wanted) -> current.revision() >= wanted ? null : wanted));
    }
    private boolean needsRefresh(SyncKey key, Instant now) {
        long wanted = required.getOrDefault(key, 0L);
        return cache.get(key, now).map(value -> value.revision() < wanted).orElse(true);
    }
    private synchronized boolean acceptCredentialRevision(long serverRevision) {
        if (credentialRevision != 0 && credentialRevision != serverRevision) { invalidate(); return false; }
        credentialRevision = serverRevision; return true;
    }
    private void invalidate() {
        cache.clear(); cursor.set(0); credentialRevision = 0;
        snapshots.values().forEach(future -> future.cancel(true)); snapshots.clear();
        required.clear();
    }
    private static void require(boolean condition) { if (!condition) throw new IllegalStateException("invalid sync response"); }
    @Override public void close() {
        if (closed.compareAndSet(false, true)) {
            scheduler.shutdownNow(); snapshots.values().forEach(future -> future.cancel(true));
            snapshots.clear(); http.close(); cache.clear();
        }
    }
    public boolean awaitClosed(Duration timeout) throws InterruptedException {
        long deadline = System.nanoTime() + timeout.toNanos();
        if (!scheduler.awaitTermination(timeout.toMillis(), TimeUnit.MILLISECONDS)) return false;
        return http.awaitClosed(Duration.ofNanos(Math.max(0, deadline - System.nanoTime())));
    }
}
