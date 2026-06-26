package com.lkjmc.common.claim;

import java.util.concurrent.atomic.AtomicReference;

public final class ClaimCache {
    private final AtomicReference<ClaimSnapshot> snapshot = new AtomicReference<>(ClaimSnapshot.empty());

    public ClaimSnapshot snapshot() {
        return snapshot.get();
    }

    public void replace(ClaimSnapshot next) {
        snapshot.set(next == null ? ClaimSnapshot.empty() : next);
    }
}
