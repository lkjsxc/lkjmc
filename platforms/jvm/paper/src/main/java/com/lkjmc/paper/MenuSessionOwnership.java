package com.lkjmc.paper;

import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;

/** Owns one synchronous menu adapter per connected player without retaining Player objects. */
public final class MenuSessionOwnership<T> {
    private final Map<UUID, T> owners = new HashMap<>();
    private boolean enabled = true;

    public synchronized void open(UUID playerId, T adapter) {
        requireEnabled();
        if (playerId == null || adapter == null) throw new IllegalArgumentException("menu owner required");
        owners.put(playerId, adapter);
    }

    public synchronized void advance(UUID playerId, T adapter) {
        requireEnabled();
        if (owners.get(playerId) != adapter) throw new IllegalStateException("menu adapter is not owned");
    }

    public synchronized Optional<T> active(UUID playerId) {
        return enabled ? Optional.ofNullable(owners.get(playerId)) : Optional.empty();
    }

    public synchronized void invalidate(UUID playerId) { owners.remove(playerId); }

    public synchronized void disable() {
        enabled = false;
        owners.clear();
    }

    public synchronized int activeOwners() { return owners.size(); }

    private void requireEnabled() {
        if (!enabled) throw new IllegalStateException("menu adapter disabled");
    }
}
