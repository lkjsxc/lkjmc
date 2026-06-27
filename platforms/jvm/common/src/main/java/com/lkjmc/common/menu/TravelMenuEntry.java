package com.lkjmc.common.menu;

public record TravelMenuEntry(String name, String serverId) {
    public TravelMenuEntry {
        if (name == null || name.isBlank()) {
            throw new IllegalArgumentException("travel entry name is required");
        }
        serverId = serverId == null ? "unknown" : serverId;
    }
}
