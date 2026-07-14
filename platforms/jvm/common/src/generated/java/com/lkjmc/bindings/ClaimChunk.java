package com.lkjmc.bindings;

import java.util.List;
import java.util.UUID;

public record ClaimChunk(
        UUID claimId,
        UUID ownerUuid,
        String ownerName,
        String name,
        String worldName,
        int chunkX,
        int chunkZ,
        List<ClaimTrust> trusts
) {
    public ClaimChunk {
        java.util.Objects.requireNonNull(claimId, "claimId");
        java.util.Objects.requireNonNull(ownerUuid, "ownerUuid");
        java.util.Objects.requireNonNull(ownerName, "ownerName");
        if (ownerName.isBlank()) throw new IllegalArgumentException("ownerName");
        java.util.Objects.requireNonNull(name, "name");
        if (name.isBlank()) throw new IllegalArgumentException("name");
        java.util.Objects.requireNonNull(worldName, "worldName");
        if (worldName.isBlank()) throw new IllegalArgumentException("worldName");
        trusts = List.copyOf(trusts);
    }
}
