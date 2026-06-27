package com.lkjmc.common.menu;

public record PlayerMenuEntry(String name) {
    public PlayerMenuEntry {
        if (name == null || name.isBlank()) {
            throw new IllegalArgumentException("player name is required");
        }
    }
}
