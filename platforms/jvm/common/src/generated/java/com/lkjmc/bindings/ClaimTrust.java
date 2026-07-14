package com.lkjmc.bindings;

import java.util.UUID;

public record ClaimTrust(
        UUID uuid,
        String name
) {
    public ClaimTrust {
        java.util.Objects.requireNonNull(uuid, "uuid");
        java.util.Objects.requireNonNull(name, "name");
        if (name.isBlank()) throw new IllegalArgumentException("name");
    }
}
