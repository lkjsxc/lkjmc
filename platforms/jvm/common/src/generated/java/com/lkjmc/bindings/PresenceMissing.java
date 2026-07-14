package com.lkjmc.bindings;

public record PresenceMissing(
        String instanceId
) implements PresencePayload {
    public PresenceMissing {
        java.util.Objects.requireNonNull(instanceId, "instanceId");
        if (instanceId.isBlank()) throw new IllegalArgumentException("instanceId");
    }
}
