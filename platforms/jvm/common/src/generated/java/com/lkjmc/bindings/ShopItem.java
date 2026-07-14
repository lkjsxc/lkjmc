package com.lkjmc.bindings;

import java.util.Map;

public record ShopItem(
        String id,
        String titleKey,
        long pricePoints,
        Map<String,String> metadata
) {
    public ShopItem {
        java.util.Objects.requireNonNull(id, "id");
        if (id.isBlank()) throw new IllegalArgumentException("id");
        java.util.Objects.requireNonNull(titleKey, "titleKey");
        if (titleKey.isBlank()) throw new IllegalArgumentException("titleKey");
        metadata = Map.copyOf(metadata);
    }
}
