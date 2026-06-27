package com.lkjmc.common.menu;

public record ShopMenuEntry(String id, String titleKey, long pricePoints, boolean deliveryAvailable) {
    public ShopMenuEntry(String id, String titleKey, long pricePoints) {
        this(id, titleKey, pricePoints, false);
    }

    public ShopMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("shop item id is required");
        }
        if (titleKey == null || titleKey.isBlank()) {
            titleKey = id;
        }
        if (pricePoints < 0) {
            throw new IllegalArgumentException("price must be non-negative");
        }
    }
}
