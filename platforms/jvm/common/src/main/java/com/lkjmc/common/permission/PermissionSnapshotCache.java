package com.lkjmc.common.permission;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import java.time.Clock;
import java.time.Duration;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;

public final class PermissionSnapshotCache {
    private static final Duration DEFAULT_TTL = Duration.ofSeconds(30);
    private final Optional<DaemonClient> client;
    private final DaemonActor actor;
    private final Duration ttl;
    private final Clock clock;
    private final PermissionResolver resolver = new PermissionResolver();
    private final ConcurrentHashMap<String, PermissionSnapshot> snapshots = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, CompletableFuture<PermissionSnapshot>> inFlight = new ConcurrentHashMap<>();

    public PermissionSnapshotCache(DaemonClient client, String actorKind, String actorName) {
        this(Optional.ofNullable(client), actorKind, actorName, DEFAULT_TTL, Clock.systemUTC());
    }

    public PermissionSnapshotCache(
        Optional<DaemonClient> client,
        String actorKind,
        String actorName,
        Duration ttl,
        Clock clock
    ) {
        this.client = client == null ? Optional.empty() : client;
        this.actor = new DaemonActor(actorKind == null ? "adapter" : actorKind,
            actorName == null || actorName.isBlank() ? "unknown" : actorName);
        this.ttl = ttl == null || ttl.isNegative() || ttl.isZero() ? DEFAULT_TTL : ttl;
        this.clock = clock == null ? Clock.systemUTC() : clock;
    }

    public static PermissionSnapshotCache disabled() {
        return new PermissionSnapshotCache(Optional.empty(), "adapter", "disabled", DEFAULT_TTL, Clock.systemUTC());
    }

    public boolean enabled() {
        return client.isPresent();
    }

    public Optional<PermissionSnapshot> snapshot(PrincipalIdentity principal) {
        if (principal == null) {
            return Optional.empty();
        }
        return Optional.ofNullable(snapshots.get(principal.cacheKey()));
    }

    public PermissionDecision decide(
        PrincipalIdentity principal,
        String permission,
        boolean platformPermission,
        boolean operator
    ) {
        if (principal != null) {
            refreshIfStale(principal);
        }
        return resolver.resolve(permission, platformPermission, operator,
            snapshot(principal).orElse(null), clock.instant());
    }

    public void refreshIfStale(PrincipalIdentity principal) {
        if (principal == null || client.isEmpty()) {
            return;
        }
        var current = snapshots.get(principal.cacheKey());
        if (current == null || !current.fresh(clock.instant())) {
            refresh(principal);
        }
    }

    public CompletableFuture<PermissionSnapshot> refresh(PrincipalIdentity principal) {
        if (principal == null || client.isEmpty()) {
            return CompletableFuture.failedFuture(new IllegalStateException("admin grant cache disabled"));
        }
        var key = principal.cacheKey();
        var future = new CompletableFuture<PermissionSnapshot>();
        var existing = inFlight.putIfAbsent(key, future);
        if (existing != null) {
            return existing;
        }
        request(principal).whenComplete((response, error) -> {
            try {
                if (error != null) {
                    future.completeExceptionally(error);
                } else if (!response.ok()) {
                    future.completeExceptionally(new IllegalStateException(response.error()
                        .map(value -> value.code()).orElse("admin.inspect_failed")));
                } else {
                    var now = clock.instant();
                    var snapshot = new PermissionSnapshot(principal, permissions(response.body()), now, now.plus(ttl));
                    snapshots.put(key, snapshot);
                    future.complete(snapshot);
                }
            } finally {
                inFlight.remove(key, future);
            }
        });
        return future;
    }

    public void evict(PrincipalIdentity principal) {
        if (principal != null) {
            snapshots.remove(principal.cacheKey());
            inFlight.remove(principal.cacheKey());
        }
    }

    private CompletableFuture<com.lkjmc.common.daemon.DaemonResponse> request(PrincipalIdentity principal) {
        return client.get().send(new DaemonRequest(UUID.randomUUID(), actor, "admin.principal.inspect", Map.of(
            "principalKind", principal.kind(),
            "principalId", principal.id(),
            "principalName", principal.name()
        )));
    }

    private static Set<String> permissions(com.google.gson.JsonObject body) {
        return DaemonJson.array(body, "permissions")
            .map(values -> {
                var result = new java.util.HashSet<String>();
                for (var value : values) {
                    if (value.isJsonPrimitive()) {
                        result.add(value.getAsString());
                    }
                }
                return Set.copyOf(result);
            })
            .orElseGet(Set::of);
    }
}
