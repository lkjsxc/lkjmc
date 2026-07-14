package com.lkjmc.bindings;

import java.util.List;

public record PluginItem(
        String id,
        String displayName,
        List<String> platforms
) {
    public PluginItem {
        java.util.Objects.requireNonNull(id, "id");
        if (id.isBlank()) throw new IllegalArgumentException("id");
        java.util.Objects.requireNonNull(displayName, "displayName");
        if (displayName.isBlank()) throw new IllegalArgumentException("displayName");
        platforms = List.copyOf(platforms);
    }
}
