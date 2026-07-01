package com.lkjmc.common.menu;

public record AchievementMenuEntry(
    String id,
    String titleKey,
    String descriptionKey,
    String category,
    String iconMaterial,
    long current,
    long required,
    String state,
    boolean hidden,
    String rewardSummary,
    String disabledReason
) {
    public AchievementMenuEntry(String id, String titleKey) {
        this(id, titleKey, titleKey + ".description", "general", "DIAMOND", 0, 1,
            "locked", false, "", "");
    }

    public AchievementMenuEntry(String id, String titleKey, long current, long required,
                                boolean claimable, boolean rewardClaimed) {
        this(id, titleKey, titleKey + ".description", "general", "DIAMOND", current, required,
            claimable ? "claimable" : rewardClaimed ? "claimed" : current > 0 ? "in-progress" : "locked",
            false, "points", claimable ? "" : "menu.achievements.disabled.not-claimable");
    }

    public AchievementMenuEntry {
        if (id == null || id.isBlank()) { throw new IllegalArgumentException("achievement id is required"); }
        if (titleKey == null || titleKey.isBlank()) { titleKey = id; }
        if (descriptionKey == null || descriptionKey.isBlank()) { descriptionKey = titleKey + ".description"; }
        if (category == null || category.isBlank()) { category = "general"; }
        if (iconMaterial == null || iconMaterial.isBlank()) { iconMaterial = "DIAMOND"; }
        if (state == null || state.isBlank()) { state = "locked"; }
        if (rewardSummary == null) { rewardSummary = ""; }
        if (disabledReason == null) { disabledReason = ""; }
        required = Math.max(1, required);
        current = Math.max(0, current);
    }

    public boolean claimable() { return "claimable".equals(state); }
    public boolean rewardClaimed() { return "claimed".equals(state); }
}
