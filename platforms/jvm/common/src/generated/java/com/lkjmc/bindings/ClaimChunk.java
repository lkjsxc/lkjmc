package com.lkjmc.bindings;

import java.util.UUID;

public record ClaimChunk(
        int chunkX,
        int chunkZ,
        UUID ownerUuid,
        String world
) {
    public ClaimChunk {
        java.util.Objects.requireNonNull(ownerUuid, "ownerUuid");
        java.util.Objects.requireNonNull(world, "world");
        if (world.isBlank()) throw new IllegalArgumentException("world");
    }
}
