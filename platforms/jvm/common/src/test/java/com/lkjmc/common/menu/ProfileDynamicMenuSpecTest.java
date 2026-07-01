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
    void achievementsSortClaimableFirstAndUseClaimPayloads() {
        var locked = new AchievementMenuEntry("locked", "achievement.locked");
        var claimable = new AchievementMenuEntry("first-home", "achievement.first-home",
            "achievement.first-home.description", "basics", "EMERALD", 1, 1, "claimable", false, "+25 points", "");
        var spec = AchievementDynamicMenus.achievements(List.of(locked, claimable));
        assertSlot(spec, 19, "achievement.first-home");
        assertEquals(new MenuAction.DaemonCommand("player.achievement.claim",
            new MenuActionPayload(java.util.Map.of("achievementId", "first-home"))), actionAt(spec, 19));
    }

    @Test
    void achievementProgressAndHiddenRowsArePolished() {
        assertEquals("[#####-----]", AchievementDynamicMenus.progressBar(5, 10));
        var hidden = new AchievementMenuEntry("secret", "achievement.secret", "achievement.secret.description",
            "secret", "DIAMOND", 0, 1, "locked", true, "", "");
        var spec = AchievementDynamicMenus.achievements(List.of(hidden));
        assertSlot(spec, 22, "menu.achievements.empty");
    }
}
