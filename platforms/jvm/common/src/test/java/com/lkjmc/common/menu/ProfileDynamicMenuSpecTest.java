package com.lkjmc.common.menu;

import static com.lkjmc.common.menu.MenuSpecAssertions.actionAt;
import static com.lkjmc.common.menu.MenuSpecAssertions.assertSlot;
import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.List;
import org.junit.jupiter.api.Test;

final class ProfileDynamicMenuSpecTest {
    @Test
    void profileSummaryUsesDaemonDataAndLinksAchievements() {
        var spec = ProfileDynamicMenus.profile(new ProfileSummary(42, 2, true));
        assertSlot(spec, 20, "menu.profile.points");
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("achievements"))), actionAt(spec, 22));
    }

    @Test
    void achievementsUseDaemonDataAsInfoRows() {
        var spec = AchievementDynamicMenus.achievements(List.of(new AchievementMenuEntry("first-home", "achievement.first-home")));
        assertSlot(spec, 19, "achievement.first-home");
        assertEquals(MenuAction.none(), actionAt(spec, 19));
    }
}
