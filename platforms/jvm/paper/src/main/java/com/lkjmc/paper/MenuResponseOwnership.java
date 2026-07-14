package com.lkjmc.paper;

import com.lkjmc.common.scheduler.PaperScheduler;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.atomic.AtomicLong;
import java.util.function.Consumer;

/** Correlates active protocol sessions without retaining Bukkit Player objects. */
public final class MenuResponseOwnership<T> {
    private final PaperScheduler scheduler;
    private final AtomicLong adapterInstances = new AtomicLong();
    private final AtomicLong generations = new AtomicLong();
    private final Map<UUID, Owned<T>> owners = new HashMap<>();
    private volatile boolean enabled = true;

    public MenuResponseOwnership(PaperScheduler scheduler) {
        if (scheduler == null) throw new IllegalArgumentException("scheduler required");
        this.scheduler = scheduler;
    }

    public synchronized Token open(UUID playerId, T adapter, long session, String locale,
                      String route, long request) {
        requireEnabled();
        var token = token(playerId, adapterInstances.incrementAndGet(), session, locale, route, request);
        owners.put(playerId, new Owned<>(adapter, token));
        return token;
    }

    public synchronized Token advance(UUID playerId, T adapter, long session, String locale,
                         String route, long request) {
        requireEnabled();
        var updated = owners.compute(playerId, (ignored, current) -> {
            if (current == null || current.adapter() != adapter)
                throw new IllegalStateException("menu adapter is not owned");
            var token = token(playerId, current.token().adapterInstance(), session, locale, route, request);
            return new Owned<>(adapter, token);
        });
        return updated.token();
    }

    public synchronized Optional<T> active(UUID playerId) {
        var owned = enabled ? owners.get(playerId) : null;
        return owned == null ? Optional.empty() : Optional.of(owned.adapter());
    }

    public synchronized Optional<T> current(Token token) {
        var owned = enabled ? owners.get(token.playerId()) : null;
        if (owned == null || !owned.token().equals(token)) return Optional.empty();
        return Optional.of(owned.adapter());
    }

    public CompletionStage<Void> onEntity(Token token, Consumer<T> completion) {
        return scheduler.entity(token.playerId(), () -> runCurrent(token, completion));
    }

    public synchronized void invalidate(UUID playerId) {
        generations.incrementAndGet();
        owners.remove(playerId);
    }

    public synchronized void invalidate(Token token) {
        var owned = owners.get(token.playerId());
        if (owned != null && owned.token().equals(token)) invalidate(token.playerId());
    }

    public synchronized void disable() {
        enabled = false;
        generations.incrementAndGet();
        owners.clear();
    }

    public synchronized int activeOwners() { return owners.size(); }

    private synchronized void runCurrent(Token token, Consumer<T> completion) {
        current(token).ifPresent(completion);
    }

    private Token token(UUID playerId, long adapterInstance, long session,
                        String locale, String route, long request) {
        return new Token(playerId, adapterInstance, generations.incrementAndGet(),
                session, request, locale, route);
    }

    private void requireEnabled() {
        if (!enabled) throw new IllegalStateException("menu adapter disabled");
    }

    public record Token(UUID playerId, long adapterInstance, long generation,
                        long session, long request, String locale, String route) {
        public Token {
            if (playerId == null || adapterInstance <= 0 || generation <= 0 || session <= 0
                    || request < 0 || locale == null || route == null)
                throw new IllegalArgumentException("complete menu ownership token required");
        }
    }

    private record Owned<T>(T adapter, Token token) { }
}
