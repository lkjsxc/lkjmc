package com.lkjmc.bindings;


public record PresenceSnapshot(
        String instanceId,
        boolean ready,
        long revision
) {
    public PresenceSnapshot {
        java.util.Objects.requireNonNull(instanceId, "instanceId");
        if (instanceId.isBlank()) throw new IllegalArgumentException("instanceId");
        if (revision <= 0) throw new IllegalArgumentException("revision");
    }
}
