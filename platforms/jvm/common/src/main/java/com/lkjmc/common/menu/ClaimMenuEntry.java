package com.lkjmc.common.menu;

public record ClaimMenuEntry(String name, long chunkCount) {
    public ClaimMenuEntry {
        if (name == null || name.isBlank()) {
            throw new IllegalArgumentException("claim name is required");
        }
        if (chunkCount < 0) {
            throw new IllegalArgumentException("chunk count must be non-negative");
        }
    }
}
