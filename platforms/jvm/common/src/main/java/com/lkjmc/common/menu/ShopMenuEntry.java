package com.lkjmc.common.menu;

public record ShopMenuEntry(
    String id,
    String titleKey,
    String category,
    String material,
    long amount,
    long pricePoints,
    String deliveryKind,
    boolean deliveryAvailable,
    boolean affordable,
    String disabledReason
) {
    public ShopMenuEntry(String id, String titleKey, long pricePoints) {
        this(id, titleKey, "misc", "CHEST", 1, pricePoints, "", false, false, "menu.disabled.shop-delivery");
    }

    public ShopMenuEntry(String id, String titleKey, long pricePoints, boolean deliveryAvailable) {
        this(id, titleKey, "misc", "CHEST", 1, pricePoints, "minecraft-item", deliveryAvailable, true,
            deliveryAvailable ? "" : "menu.disabled.shop-delivery");
    }

    public ShopMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("shop item id is required");
        }
        if (titleKey == null || titleKey.isBlank()) { titleKey = id; }
        if (category == null || category.isBlank()) { category = "misc"; }
        if (material == null || material.isBlank()) { material = "CHEST"; }
        if (deliveryKind == null) { deliveryKind = ""; }
        if (disabledReason == null) { disabledReason = ""; }
        if (pricePoints < 0 || amount < 0) {
            throw new IllegalArgumentException("shop price and amount must be non-negative");
        }
    }
}
