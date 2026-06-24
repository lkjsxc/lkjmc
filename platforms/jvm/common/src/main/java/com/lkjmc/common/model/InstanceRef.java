package com.lkjmc.common.model;

public record InstanceRef(String id, String kind) {
    public InstanceRef {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("instance id is required");
        }
    }
}
