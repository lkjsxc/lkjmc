package com.lkjmc.common.sync;

import java.time.Duration;
import java.util.function.LongSupplier;

final class RetryGate {
    private final ReconnectBackoff backoff;
    private final SyncKey key;
    private final LongSupplier clock;
    private int failures;
    private long nextAttempt;

    RetryGate(ReconnectBackoff backoff, SyncKey key, LongSupplier clock) {
        this.backoff = backoff;
        this.key = key;
        this.clock = clock;
    }

    synchronized boolean canAttempt() {
        return clock.getAsLong() >= nextAttempt;
    }

    synchronized Duration failed() {
        failures = Math.min(failures + 1, 17);
        Duration delay = backoff.delay(failures, key);
        long now = clock.getAsLong();
        nextAttempt = now > Long.MAX_VALUE - delay.toNanos()
                ? Long.MAX_VALUE : now + delay.toNanos();
        return delay;
    }

    synchronized void succeeded() {
        failures = 0;
        nextAttempt = 0;
    }

    synchronized int failures() {
        return failures;
    }
}
