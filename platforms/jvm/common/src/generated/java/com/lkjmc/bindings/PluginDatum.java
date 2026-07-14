package com.lkjmc.bindings;

public record PluginDatum(
        String key,
        String value
) {
    public PluginDatum {
        java.util.Objects.requireNonNull(key, "key");
        if (key.isBlank()) throw new IllegalArgumentException("key");
        java.util.Objects.requireNonNull(value, "value");
        if (value.isBlank()) throw new IllegalArgumentException("value");
    }
}
