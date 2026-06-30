package com.lkjmc.common.menu;

public record AdventureMenuEntry(
    String id,
    String titleKey,
    String iconMaterial,
    long pricePoints,
    int maxPartySize,
    boolean enabled
) {
    public AdventureMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("adventure id is required");
        }
        if (titleKey == null || titleKey.isBlank()) {
            titleKey = id;
        }
        if (iconMaterial == null || iconMaterial.isBlank()) {
            iconMaterial = "MAP";
        }
    }
}
