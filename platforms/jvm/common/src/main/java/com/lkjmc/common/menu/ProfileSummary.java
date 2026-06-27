package com.lkjmc.common.menu;

public record ProfileSummary(long pointsBalance, int achievementCount, boolean loaded) {
    public static ProfileSummary loading() {
        return new ProfileSummary(0, 0, false);
    }
}
