package com.lkjmc.common.menu;

public record DailyRewardStatus(boolean claimedToday, long points, boolean loaded) {
    public static DailyRewardStatus loading() {
        return new DailyRewardStatus(false, 0, false);
    }
}
