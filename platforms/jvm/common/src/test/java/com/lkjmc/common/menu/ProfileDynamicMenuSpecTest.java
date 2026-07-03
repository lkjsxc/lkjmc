package com.lkjmc.common.menu;

import static com.lkjmc.common.menu.MenuSpecAssertions.actionAt;
import static com.lkjmc.common.menu.MenuSpecAssertions.assertSlot;
import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

final class ProfileDynamicMenuSpecTest {
    @Test
    void profileSummaryUsesDaemonDataAndLinksAchievements() {
        var spec = ProfileDynamicMenus.profile(new ProfileSummary(42, 2, true));
        assertSlot(spec, 20, "menu.profile.points");
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("achievements"))), actionAt(spec, 22));
    }

    @Test
    void achievementsUseBrowserDirectoriesAndDetailClaims() {
        var claimable = new AchievementMenuEntry("first-home", "achievement.first-home",
            "achievement.first-home.description", "getting-started", "EMERALD", 1, 1,
            "claimable", false, "+25 points", "");
        var root = AchievementDynamicMenus.root(List.of(claimable));
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("achievement-directory"),
            Map.of("path", "claimable"))), actionAt(root, 19));
        var directory = AchievementDynamicMenus.directory(List.of(claimable), "claimable");
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("achievement-detail"),
            Map.of("id", "first-home"))), actionAt(directory, 19));
        var detail = AchievementDynamicMenus.detail(List.of(claimable), "first-home");
        assertEquals(new MenuAction.DaemonCommand("player.achievement.claim",
            new MenuActionPayload(Map.of("achievementId", "first-home"))), actionAt(detail, 31));
    }

    @Test
    void achievementProgressAndHiddenRowsArePolished() {
        assertEquals("[#####-----]", AchievementDynamicMenus.progressBar(5, 10));
        var hidden = new AchievementMenuEntry("secret", "achievement.secret", "achievement.secret.description",
            "secret", "DIAMOND", 0, 1, "locked", true, "", "");
        var spec = AchievementDynamicMenus.root(List.of(hidden));
        assertSlot(spec, 22, "menu.achievements.empty");
    }
}
