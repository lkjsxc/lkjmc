package com.lkjmc.common.daemon;

public record DaemonActor(String kind, String name) {
    public DaemonActor {
        if (kind == null || kind.isBlank() || name == null || name.isBlank()) {
            throw new IllegalArgumentException("actor kind and name are required");
        }
    }
}
