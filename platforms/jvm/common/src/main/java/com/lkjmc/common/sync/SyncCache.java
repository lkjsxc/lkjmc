package com.lkjmc.common.sync;

import java.time.Duration;
import java.time.Instant;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Optional;

public final class SyncCache {
    private final int maxEntries;
    private final long maxBytes;
    private final Duration maxAge;
    private final LinkedHashMap<SyncKey, SyncSnapshot> entries = new LinkedHashMap<>(16, 0.75f, true);
    private long bytes;

    public SyncCache(int maxEntries, long maxBytes, Duration maxAge) {
        if (maxEntries < 1 || maxBytes < 1 || maxAge.isNegative() || maxAge.isZero()) {
            throw new IllegalArgumentException("invalid cache bounds");
        }
        this.maxEntries = maxEntries;
        this.maxBytes = maxBytes;
        this.maxAge = maxAge;
    }

    public synchronized boolean put(SyncSnapshot candidate, Instant now) {
        expire(now);
        var current = entries.get(candidate.key());
        if (current != null && candidate.revision() <= current.revision()) {
            return false;
        }
        if (candidate.encodedBytes() > maxBytes) {
            return false;
        }
        if (current != null) {
            bytes -= current.encodedBytes();
        }
        entries.put(candidate.key(), candidate);
        bytes += candidate.encodedBytes();
        trim();
        return entries.get(candidate.key()) == candidate;
    }

    public synchronized Optional<SyncSnapshot> get(SyncKey key, Instant now) {
        expire(now);
        return Optional.ofNullable(entries.get(key));
    }

    public synchronized void clear() {
        entries.clear();
        bytes = 0;
    }

    public synchronized int size() {
        return entries.size();
    }

    public synchronized long bytes() {
        return bytes;
    }

    private void expire(Instant now) {
        entries.entrySet().removeIf(entry -> {
            boolean expired = Duration.between(entry.getValue().receivedAt(), now).compareTo(maxAge) > 0;
            if (expired) {
                bytes -= entry.getValue().encodedBytes();
            }
            return expired;
        });
    }

    private void trim() {
        while (entries.size() > maxEntries || bytes > maxBytes) {
            Map.Entry<SyncKey, SyncSnapshot> eldest = entries.entrySet().iterator().next();
            bytes -= eldest.getValue().encodedBytes();
            entries.remove(eldest.getKey());
        }
    }
}
