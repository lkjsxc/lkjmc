package com.lkjmc.common.menu;

public record MenuRouteParam(String key, String value) {
    public MenuRouteParam {
        if (key == null || key.isBlank()) {
            throw new IllegalArgumentException("route param key is required");
        }
        value = value == null ? "" : value;
    }
}
