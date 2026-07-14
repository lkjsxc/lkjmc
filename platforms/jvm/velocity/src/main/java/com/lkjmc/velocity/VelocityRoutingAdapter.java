package com.lkjmc.velocity;

import com.lkjmc.bindings.Route;
import com.lkjmc.bindings.RoutingSnapshot;
import com.lkjmc.common.scheduler.VelocityScheduler;
import java.time.Instant;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CompletionStage;
import java.util.stream.Collectors;

public final class VelocityRoutingAdapter {
    public static final String OWNED_PREFIX = "lkjmc-owned-";
    private final RoutingPlatform platform;
    private final VelocityScheduler scheduler;

    public VelocityRoutingAdapter(RoutingPlatform platform, VelocityScheduler scheduler) {
        this.platform = platform;
        this.scheduler = scheduler;
    }

    public CompletionStage<Boolean> reconcile(
            RoutingSnapshot snapshot,
            long requiredRevision,
            Instant now) {
        if (snapshot == null || now == null || snapshot.revision() != requiredRevision
                || !snapshot.expiresAt().isAfter(now)) {
            return java.util.concurrent.CompletableFuture.completedFuture(false);
        }
        Map<String, Route> desired = snapshot.routes().stream().filter(Route::ready)
                .collect(Collectors.toUnmodifiableMap(route -> owned(route.id()), route -> route));
        return scheduler.event(() -> apply(desired)).thenApply(unused -> verify(desired))
                .exceptionally(failure -> false);
    }

    private void apply(Map<String, Route> desired) {
        Set<String> actual = platform.registrations();
        desired.forEach((name, route) -> {
            if (actual.contains(name) && !platform.route(name).filter(route::equals).isPresent()
                    && !platform.unregister(name)) {
                throw new IllegalStateException("registration replacement unavailable");
            }
            if (!platform.registrations().contains(name) && !platform.register(name, route)) {
                throw new IllegalStateException("registration unavailable");
            }
        });
        actual.stream().filter(this::isOwned).filter(name -> !desired.containsKey(name))
                .forEach(name -> {
                    if (!platform.unregister(name)) throw new IllegalStateException("unregister unavailable");
                });
    }

    private boolean verify(Map<String, Route> desired) {
        Set<String> actualOwned = platform.registrations().stream().filter(this::isOwned)
                .collect(Collectors.toUnmodifiableSet());
        return actualOwned.equals(desired.keySet())
                && desired.entrySet().stream().allMatch(entry -> platform.route(entry.getKey())
                    .filter(entry.getValue()::equals).isPresent());
    }

    private boolean isOwned(String name) {
        return name.startsWith(OWNED_PREFIX);
    }

    public static String owned(String routeId) {
        if (routeId == null || !routeId.matches("[A-Za-z0-9._-]{1,96}")) {
            throw new IllegalArgumentException("invalid route id");
        }
        return OWNED_PREFIX + routeId;
    }
}
