package com.lkjmc.common.menu;

import java.util.Map;

public record MenuRoute(MenuId id, Map<String, String> params) {
    public MenuRoute(MenuId id) {
        this(id, Map.of());
    }

    public MenuRoute {
        if (id == null) {
            throw new IllegalArgumentException("route id is required");
        }
        params = Map.copyOf(params == null ? Map.of() : params);
    }
}
