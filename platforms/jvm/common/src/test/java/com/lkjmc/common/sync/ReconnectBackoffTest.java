package com.lkjmc.common.sync;

import static org.junit.jupiter.api.Assertions.*;

import java.time.Duration;
import org.junit.jupiter.api.Test;

final class ReconnectBackoffTest {
    @Test
    void delayIsDeterministicCappedAndKeyJittered() {
        ReconnectBackoff backoff = new ReconnectBackoff(Duration.ofMillis(100), Duration.ofSeconds(2));
        SyncKey first = new SyncKey("routing", "network");
        SyncKey second = new SyncKey("presence", "hub");
        assertEquals(backoff.delay(4, first), backoff.delay(4, first));
        assertNotEquals(backoff.delay(4, first), backoff.delay(4, second));
        assertTrue(backoff.delay(100, first).compareTo(Duration.ofSeconds(2)) <= 0);
        assertTrue(backoff.delay(1, first).compareTo(Duration.ofMillis(100)) >= 0);
    }
}
