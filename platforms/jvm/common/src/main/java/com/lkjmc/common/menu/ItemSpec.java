package com.lkjmc.common.menu;

import java.util.List;

public record ItemSpec(String material, String nameKey, List<String> loreKeys) {
    public ItemSpec {
        if (material == null || material.isBlank()) {
            throw new IllegalArgumentException("material is required");
        }
        loreKeys = List.copyOf(loreKeys == null ? List.of() : loreKeys);
    }
}
