package com.lkjmc.paper.ui;

import com.lkjmc.common.ui.kernel.MenuRoute;
import com.lkjmc.common.ui.kernel.RouteView;
import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;
import java.util.UUID;

public final class UiStaleCache {
    static final Duration DEFAULT_TTL = Duration.ofMinutes(2);
    private static final int MAX_ROUTES_PER_PLAYER = 32;
    private final Duration ttl;
    private final Clock clock;
    private final Map<UUID, LinkedHashMap<String, Entry>> entries = new LinkedHashMap<>();

    public UiStaleCache() {
        this(DEFAULT_TTL, Clock.systemUTC());
    }

    UiStaleCache(Duration ttl, Clock clock) {
        this.ttl = ttl == null || ttl.isNegative() || ttl.isZero() ? DEFAULT_TTL : ttl;
        this.clock = clock == null ? Clock.systemUTC() : clock;
    }

    public synchronized void remember(UUID player, MenuRoute route, RouteView view) {
        if (player == null || route == null || view == null) {
            return;
        }
        var playerEntries = entries.computeIfAbsent(player, ignored -> new LinkedHashMap<>());
        prune(playerEntries, now());
        playerEntries.put(key(route), new Entry(view, now()));
        while (playerEntries.size() > MAX_ROUTES_PER_PLAYER) {
            var first = playerEntries.keySet().iterator().next();
            playerEntries.remove(first);
        }
    }

    public synchronized Optional<RouteView> find(UUID player, MenuRoute route) {
        var playerEntries = entries.get(player);
        if (playerEntries == null || route == null) {
            return Optional.empty();
        }
        var current = now();
        prune(playerEntries, current);
        var entry = playerEntries.get(key(route));
        if (entry == null || expired(entry, current)) {
            playerEntries.remove(key(route));
            return Optional.empty();
        }
        return Optional.of(entry.view());
    }

    synchronized int size(UUID player) {
        var playerEntries = entries.get(player);
        if (playerEntries == null) {
            return 0;
        }
        prune(playerEntries, now());
        return playerEntries.size();
    }

    private void prune(Map<String, Entry> playerEntries, Instant current) {
        playerEntries.entrySet().removeIf(entry -> expired(entry.getValue(), current));
    }

    private boolean expired(Entry entry, Instant current) {
        return entry.loadedAt().plus(ttl).isBefore(current);
    }

    private Instant now() {
        return clock.instant();
    }

    private static String key(MenuRoute route) {
        return route.id() + "?" + new TreeMap<>(route.params());
    }

    private record Entry(RouteView view, Instant loadedAt) {}
}
