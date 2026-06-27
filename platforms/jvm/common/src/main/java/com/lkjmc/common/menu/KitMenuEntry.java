package com.lkjmc.common.menu;

public record KitMenuEntry(String id, String titleKey, long rewardPoints, long cooldownHours) {
    public KitMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("kit id is required");
        }
        if (titleKey == null || titleKey.isBlank()) {
            titleKey = id;
        }
    }
}
