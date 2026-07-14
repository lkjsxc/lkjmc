package com.lkjmc.common.sync;

import static org.junit.jupiter.api.Assertions.*;

import com.lkjmc.bindings.ClaimPayload;
import com.lkjmc.bindings.ClaimSnapshot;
import com.lkjmc.bindings.PresenceMissing;
import com.lkjmc.bindings.PresenceSnapshot;
import java.time.Duration;
import java.time.Instant;
import java.util.List;
import org.junit.jupiter.api.Test;

final class SyncCacheTest {
    private static final SyncKey KEY = new SyncKey("claims", "survival");

    @Test
    void rejectsReorderedAndExpiresOldSnapshots() {
        Instant now = Instant.parse("2026-07-14T00:00:00Z");
        SyncCache cache = new SyncCache(2, 100, Duration.ofSeconds(5));
        assertTrue(cache.put(snapshot(2, 10, now), now));
        assertFalse(cache.put(snapshot(1, 10, now), now));
        assertEquals(2, cache.get(KEY, now).orElseThrow().revision());
        assertTrue(cache.get(KEY, now.plusSeconds(6)).isEmpty());
    }

    @Test
    void boundsEntriesAndBytes() {
        Instant now = Instant.parse("2026-07-14T00:00:00Z");
        SyncCache cache = new SyncCache(1, 15, Duration.ofMinutes(1));
        assertTrue(cache.put(snapshot(1, 10, now), now));
        var value = new PresenceSnapshot("presence", "hub", 1, now, 1, new PresenceMissing("hub"));
        assertTrue(cache.put(new SyncSnapshot(value, 10, now), now));
        assertEquals(1, cache.size());
        assertTrue(cache.get(KEY, now).isEmpty());
        assertFalse(cache.put(snapshot(2, 16, now), now));
        assertTrue(cache.bytes() <= 15);
    }

    private static SyncSnapshot snapshot(long revision, int bytes, Instant now) {
        var value = new ClaimSnapshot("claims", "survival", revision, now, 1,
                new ClaimPayload(List.of()));
        return new SyncSnapshot(value, bytes, now);
    }
}
