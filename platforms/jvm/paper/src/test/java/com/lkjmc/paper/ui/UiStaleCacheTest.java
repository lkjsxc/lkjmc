package com.lkjmc.paper.ui;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.MenuRoute;
import com.lkjmc.common.ui.kernel.RouteView;
import com.lkjmc.common.ui.kernel.TextRef;
import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.ZoneId;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.Test;

final class UiStaleCacheTest {
    private static final UUID PLAYER = UiTestFixtures.PLAYER_ID;

    @Test
    void boundsRoutesPerPlayer() {
        var clock = new MutableClock();
        var cache = new UiStaleCache(Duration.ofMinutes(2), clock);
        for (int i = 0; i < 33; i++) {
            cache.remember(PLAYER, new MenuRoute("route-" + i), view("v" + i));
        }

        assertEquals(32, cache.size(PLAYER));
        assertTrue(cache.find(PLAYER, new MenuRoute("route-0")).isEmpty());
        assertTrue(cache.find(PLAYER, new MenuRoute("route-32")).isPresent());
    }

    @Test
    void expiresEntries() {
        var clock = new MutableClock();
        var cache = new UiStaleCache(Duration.ofSeconds(1), clock);
        var route = new MenuRoute("shop", Map.of("category", "all"));
        cache.remember(PLAYER, route, view("shop"));

        clock.advance(Duration.ofSeconds(2));

        assertTrue(cache.find(PLAYER, route).isEmpty());
        assertEquals(0, cache.size(PLAYER));
    }

    private static RouteView view(String value) {
        return new RouteView.ListView(List.of(new EntryView("STONE", TextRef.literal(value),
            List.of(), com.lkjmc.common.ui.document.ItemRole.INFO,
            new com.lkjmc.common.ui.document.DocumentAction.None())), List.of());
    }

    private static final class MutableClock extends Clock {
        private Instant now = Instant.parse("2026-01-01T00:00:00Z");
        @Override public ZoneId getZone() { return ZoneId.of("UTC"); }
        @Override public Clock withZone(ZoneId zone) { return this; }
        @Override public Instant instant() { return now; }
        void advance(Duration duration) { now = now.plus(duration); }
    }
}
