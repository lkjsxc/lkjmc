package com.lkjmc.common.menu;

public record ServerMenuEntry(
    String id,
    String kind,
    String desiredState,
    String observedState,
    boolean healthy,
    Integer playerCount
) {
    public ServerMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("server id is required");
        }
        kind = kind == null ? "unknown" : kind;
        desiredState = desiredState == null ? "unknown" : desiredState;
        observedState = observedState == null ? "unknown" : observedState;
    }
}
