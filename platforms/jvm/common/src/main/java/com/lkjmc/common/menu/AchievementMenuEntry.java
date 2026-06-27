package com.lkjmc.common.menu;

public record AchievementMenuEntry(String id, String titleKey) {
    public AchievementMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("achievement id is required");
        }
        if (titleKey == null || titleKey.isBlank()) {
            titleKey = id;
        }
    }
}
