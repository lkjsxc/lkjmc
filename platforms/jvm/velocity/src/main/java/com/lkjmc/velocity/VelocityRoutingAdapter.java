package com.lkjmc.velocity;

import com.lkjmc.bindings.RoutingInstance;
import com.lkjmc.bindings.RoutingSnapshot;
import com.lkjmc.common.scheduler.VelocityScheduler;
import java.time.Instant;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
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
                || !"routing".equals(snapshot.domain()) || !"network".equals(snapshot.key())
                || snapshot.generatedAt().isAfter(now)) {
            return CompletableFuture.completedFuture(false);
        }
        Map<String, RoutingTarget> desired;
        try {
            desired = targets(snapshot);
        } catch (RuntimeException malformed) {
            return CompletableFuture.completedFuture(false);
        }
        return scheduler.event(() -> apply(desired)).thenApply(unused -> verify(desired))
                .exceptionally(failure -> false);
    }

    private Map<String, RoutingTarget> targets(RoutingSnapshot snapshot) {
        Map<String, RoutingTarget> result = new LinkedHashMap<>();
        for (RoutingInstance instance : snapshot.payload().instances()) {
            if (!Boolean.TRUE.equals(instance.ready())) continue;
            var ports = instance.ports().stream().filter(port -> "minecraft".equals(port.purpose())).toList();
            if (ports.size() != 1) throw new IllegalArgumentException("one Minecraft port required");
            String name = owned(instance.id());
            if (result.put(name, new RoutingTarget("127.0.0.1", instance.id(), ports.getFirst().port())) != null) {
                throw new IllegalArgumentException("duplicate route");
            }
        }
        return Map.copyOf(result);
    }

    private void apply(Map<String, RoutingTarget> desired) {
        Set<String> actual = platform.registrations();
        desired.forEach((name, route) -> {
            if (actual.contains(name) && platform.route(name).filter(route::equals).isEmpty()
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

    private boolean verify(Map<String, RoutingTarget> desired) {
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
