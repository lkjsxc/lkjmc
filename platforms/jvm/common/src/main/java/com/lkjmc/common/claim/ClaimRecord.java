package com.lkjmc.common.claim;

import java.util.Set;

public record ClaimRecord(
    String claimId,
    String ownerUuid,
    String ownerName,
    String name,
    ClaimChunk chunk,
    Set<String> trustedUuids
) {
    public ClaimRecord {
        if (claimId == null || claimId.isBlank() || ownerUuid == null || ownerUuid.isBlank() || chunk == null) {
            throw new IllegalArgumentException("claim id, owner uuid, and chunk are required");
        }
        ownerName = ownerName == null ? "" : ownerName;
        name = name == null ? "" : name;
        trustedUuids = Set.copyOf(trustedUuids == null ? Set.of() : trustedUuids);
    }
}
