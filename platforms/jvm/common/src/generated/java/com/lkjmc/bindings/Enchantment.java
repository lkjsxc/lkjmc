package com.lkjmc.bindings;

public record Enchantment(
        String id,
        int level
) {
    public Enchantment {
        java.util.Objects.requireNonNull(id, "id");
        if (id.isBlank()) throw new IllegalArgumentException("id");
    }
}
