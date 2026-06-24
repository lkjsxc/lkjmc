package com.lkjmc.common.menu;

public record MenuTitle(String key) {
    public MenuTitle {
        if (key == null || key.isBlank()) {
            throw new IllegalArgumentException("menu title key is required");
        }
    }
}
