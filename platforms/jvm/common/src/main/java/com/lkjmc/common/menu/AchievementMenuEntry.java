package com.lkjmc.common.menu;

public record AchievementMenuEntry(
    String id,
    String titleKey,
    long current,
    long required,
    boolean claimable,
    boolean rewardClaimed
) {
    public AchievementMenuEntry(String id, String titleKey) {
        this(id, titleKey, 0, 1, false, false);
    }

    public AchievementMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("achievement id is required");
        }
        if (titleKey == null || titleKey.isBlank()) {
            titleKey = id;
        }
        required = Math.max(1, required);
        current = Math.max(0, current);
    }
}
