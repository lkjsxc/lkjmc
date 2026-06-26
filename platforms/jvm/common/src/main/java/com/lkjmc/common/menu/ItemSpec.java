package com.lkjmc.common.menu;

import java.util.List;

public record ItemSpec(String material, String nameKey, List<String> loreKeys, ItemVisualRole role) {
    public ItemSpec(String material, String nameKey, List<String> loreKeys) {
        this(material, nameKey, loreKeys, ItemVisualRole.ACTION);
    }

    public ItemSpec {
        if (material == null || material.isBlank()) {
            throw new IllegalArgumentException("material is required");
        }
        if (nameKey == null || nameKey.isBlank()) {
            throw new IllegalArgumentException("name key is required");
        }
        loreKeys = List.copyOf(loreKeys == null ? List.of() : loreKeys);
        role = role == null ? ItemVisualRole.ACTION : role;
    }

    public boolean inert() {
        return role == ItemVisualRole.INFO || role == ItemVisualRole.DECORATION;
    }
}
