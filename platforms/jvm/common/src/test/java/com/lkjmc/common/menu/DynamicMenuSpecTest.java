package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.List;
import org.junit.jupiter.api.Test;

final class DynamicMenuSpecTest {
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

    @Test
    void reportListUsesDaemonDataAndDisabledModeration() {
        var spec = ReportDynamicMenus.reports(List.of(new ReportMenuEntry("12345678-aaaa", "hub", "grief", "open")));
        assertSlot(spec, 19, "literal:report 12345678");
        assertEquals(new MenuAction.Disabled("menu.disabled.report-confirmation"), actionAt(spec, 19));
    }

    @Test
    void reportListCanRenderPermissionDeniedWithoutDaemonData() {
        var spec = ReportDynamicMenus.reports(List.of(), false);
        assertSlot(spec, 22, "menu.reports.denied");
        assertEquals(new MenuAction.Disabled("menu.disabled.reports-permission"), actionAt(spec, 22));
    }

    @Test
    void mailInboxUsesDaemonDataAndReadCommands() {
        var id = "00000000-0000-0000-0000-000000000000";
        var spec = MailDynamicMenus.mail(List.of(new MailMenuEntry(id, "Alex", "hi", false)));
        assertSlot(spec, 19, "literal:Alex");
        assertEquals(new MenuAction.RunPlayerCommand("mail read " + id), actionAt(spec, 19));
    }

    @Test
    void voteListUsesDaemonDataAndDisabledLinks() {
        var spec = VoteDynamicMenus.votes(List.of(new VoteMenuEntry("site", "vote.site", "https://example.test")));
        assertSlot(spec, 19, "vote.site");
        assertEquals(new MenuAction.Disabled("menu.disabled.vote-open"), actionAt(spec, 19));
    }

    @Test
    void dailyStatusEnablesOnlyUnclaimedRewards() {
        var ready = DailyDynamicMenus.daily(new DailyRewardStatus(false, 100, true));
        assertEquals(new MenuAction.RunPlayerCommand("daily"), actionAt(ready, 22));
        var claimed = DailyDynamicMenus.daily(new DailyRewardStatus(true, 100, true));
        assertEquals(new MenuAction.Disabled("menu.disabled.daily-claimed"), actionAt(claimed, 22));
    }

    @Test
    void kitListUsesDaemonDataAndClaimCommands() {
        var spec = KitDynamicMenus.kits(List.of(new KitMenuEntry("daily", "kit.daily", 10, 24)));
        assertSlot(spec, 19, "kit.daily");
        assertEquals(new MenuAction.RunPlayerCommand("kit claim daily"), actionAt(spec, 19));
    }

    @Test
    void shopListUsesDaemonDataAndDisablesUndeliverablePurchases() {
        var spec = ShopDynamicMenus.shop(List.of(new ShopMenuEntry("apple", "shop.apple", 5)));
        assertSlot(spec, 19, "shop.apple");
        assertEquals(new MenuAction.Disabled("menu.disabled.shop-delivery"), actionAt(spec, 19));
    }

    @Test
    void claimListUsesDaemonDataAndDisabledDetails() {
        var spec = ClaimDynamicMenus.claims(List.of(new ClaimMenuEntry("base", 2)));
        assertSlot(spec, 19, "literal:base");
        assertEquals(new MenuAction.Disabled("menu.disabled.claim-detail"), actionAt(spec, 19));
    }

    @Test
    void travelListsUseDaemonDataAndCommandPayloads() {
        var spec = TravelDynamicMenus.homes(List.of(new TravelMenuEntry("zeta", "hub"), new TravelMenuEntry("alpha", "survival")));
        assertSlot(spec, 19, "literal:alpha");
        assertEquals(new MenuAction.RunPlayerCommand("home alpha"), actionAt(spec, 19));
    }

    @Test
    void dynamicServerListUsesStableSlots() {
        var spec = DynamicMenus.serverList(List.of(
            new ServerMenuEntry("zeta", "folia", "running", "process-healthy", true, 3),
            new ServerMenuEntry("alpha", "purpur", "suspended", "process-absent", false, 0)
        ));
        assertSlot(spec, 19, "literal:alpha · suspended");
        assertSlot(spec, 20, "literal:zeta · running");
        assertEquals(new MenuAction.Disabled("menu.disabled.server-start-permission"), actionAt(spec, 19));
    }

    @Test
    void dynamicServerListAllowsSafeLifecycleCommands() {
        var permissions = new ServerMenuPermissions(true, true);
        var spec = DynamicMenus.serverList(List.of(
            new ServerMenuEntry("alpha", "purpur", "suspended", "process-absent", false, 0),
            new ServerMenuEntry("beta", "folia", "running", "process-healthy", true, 0),
            new ServerMenuEntry("gamma", "folia", "running", "process-healthy", true, 2)
        ), permissions);
        assertEquals(new MenuAction.RunPlayerCommand("lkjmc server start alpha"), actionAt(spec, 19));
        assertEquals(new MenuAction.RunPlayerCommand("lkjmc server stop beta"), actionAt(spec, 20));
        assertEquals(new MenuAction.Disabled("menu.disabled.server-occupied"), actionAt(spec, 21));
    }

    private static MenuAction actionAt(MenuSpec spec, int slot) {
        return spec.slots().stream().filter(value -> value.slot() == slot).findFirst().orElseThrow().action();
    }

    private static void assertSlot(MenuSpec spec, int slot, String key) {
        assertEquals(key, spec.slots().stream().filter(value -> value.slot() == slot).findFirst().orElseThrow().item().nameKey());
    }
}
