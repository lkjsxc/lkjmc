package com.lkjmc.bindings;


public record ItemSlot(
        int amount,
        String material,
        int slot
) {
    public ItemSlot {
        java.util.Objects.requireNonNull(material, "material");
        if (material.isBlank()) throw new IllegalArgumentException("material");
    }
}
