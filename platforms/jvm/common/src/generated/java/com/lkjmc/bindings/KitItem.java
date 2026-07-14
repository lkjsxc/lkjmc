package com.lkjmc.bindings;

public record KitItem(
        String id,
        String titleKey,
        long rewardPoints,
        long cooldownHours
) {
    public KitItem {
        java.util.Objects.requireNonNull(id, "id");
        if (id.isBlank()) throw new IllegalArgumentException("id");
        java.util.Objects.requireNonNull(titleKey, "titleKey");
        if (titleKey.isBlank()) throw new IllegalArgumentException("titleKey");
    }
}
