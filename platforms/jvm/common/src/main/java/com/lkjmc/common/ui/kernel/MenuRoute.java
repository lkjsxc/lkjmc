package com.lkjmc.common.ui.kernel;

import java.util.Map;
import java.util.TreeMap;

public record MenuRoute(String id, Map<String, String> params) {
    public MenuRoute(String id) {
        this(id, Map.of());
    }

    public MenuRoute {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("route id is required");
        }
        params = Map.copyOf(params == null ? Map.of() : new TreeMap<>(params));
    }

    public static MenuRoute root() {
        return new MenuRoute("root");
    }

    public boolean isRoot() {
        return "root".equals(id);
    }
}
