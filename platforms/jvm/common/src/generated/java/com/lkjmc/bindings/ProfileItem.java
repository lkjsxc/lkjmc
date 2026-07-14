package com.lkjmc.bindings;

import java.util.List;

public record ProfileItem(
        String material,
        int amount,
        long damage,
        String customName,
        List<String> lore,
        List<Enchantment> enchantments,
        Integer customModelData
) {
    public ProfileItem {
        java.util.Objects.requireNonNull(material, "material");
        if (material.isBlank()) throw new IllegalArgumentException("material");
        lore = List.copyOf(lore);
        enchantments = List.copyOf(enchantments);
    }
}
