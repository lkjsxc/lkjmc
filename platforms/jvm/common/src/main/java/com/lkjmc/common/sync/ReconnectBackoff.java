package com.lkjmc.common.sync;

import java.time.Duration;

public final class ReconnectBackoff {
    private final Duration base;
    private final Duration cap;

    public ReconnectBackoff(Duration base, Duration cap) {
        if (base.isNegative() || base.isZero() || cap.compareTo(base) < 0) {
            throw new IllegalArgumentException("invalid backoff bounds");
        }
        this.base = base;
        this.cap = cap;
    }

    public Duration delay(int failures, SyncKey key) {
        int exponent = Math.min(Math.max(failures - 1, 0), 16);
        long raw = Math.min(cap.toMillis(), Math.multiplyExact(base.toMillis(), 1L << exponent));
        long spread = Math.max(1, raw / 5);
        long jitter = Math.floorMod(key.hashCode() * 1103515245L, spread * 2 + 1) - spread;
        return Duration.ofMillis(Math.max(base.toMillis(), Math.min(cap.toMillis(), raw + jitter)));
    }
}
