package com.lkjmc.bindings;

import java.time.Instant;

public record PresenceAvailable(
        String instanceId,
        Integer playerCount,
        Integer maxPlayers,
        boolean ready,
        Instant lastHeartbeatAt,
        String suspendReason
) implements PresencePayload {
    public PresenceAvailable {
        java.util.Objects.requireNonNull(instanceId, "instanceId");
        if (instanceId.isBlank()) throw new IllegalArgumentException("instanceId");
        java.util.Objects.requireNonNull(lastHeartbeatAt, "lastHeartbeatAt");
    }
}
