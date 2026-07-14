package com.lkjmc.common.sync;

import java.time.Duration;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class RetryGateTest {
    @Test
    void deterministic_clock_enforces_backoff_and_success_reset() {
        AtomicLong clock = new AtomicLong();
        RetryGate gate = new RetryGate(new ReconnectBackoff(
                Duration.ofMillis(200), Duration.ofSeconds(10)),
                new SyncKey("settings", "4a1f2b5c-2a1e-4c7a-8b6d-111111111111"), clock::get);
        assertTrue(gate.canAttempt());
        Duration first = gate.failed();
        assertEquals(1, gate.failures());
        assertFalse(gate.canAttempt());
        clock.addAndGet(first.toNanos() - 1);
        assertFalse(gate.canAttempt());
        clock.incrementAndGet();
        assertTrue(gate.canAttempt());
        Duration second = gate.failed();
        assertTrue(second.compareTo(first) > 0);
        clock.addAndGet(second.toNanos());
        gate.succeeded();
        assertEquals(0, gate.failures());
        assertTrue(gate.canAttempt());
    }
}
