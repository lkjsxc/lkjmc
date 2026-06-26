package com.lkjmc.common.claim;

public record ClaimChunk(String instanceId, String worldName, int chunkX, int chunkZ) {
    public ClaimChunk {
        if (instanceId == null || instanceId.isBlank() || worldName == null || worldName.isBlank()) {
            throw new IllegalArgumentException("instance and world are required");
        }
    }
}
