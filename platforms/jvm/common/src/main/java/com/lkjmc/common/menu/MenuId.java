package com.lkjmc.common.menu;

public record MenuId(String value) {
    public MenuId {
        if (value == null || value.isBlank()) {
            throw new IllegalArgumentException("menu id is required");
        }
    }
}
