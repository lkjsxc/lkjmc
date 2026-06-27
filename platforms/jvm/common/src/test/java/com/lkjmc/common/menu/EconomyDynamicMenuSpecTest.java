package com.lkjmc.common.menu;

import static com.lkjmc.common.menu.MenuSpecAssertions.actionAt;
import static com.lkjmc.common.menu.MenuSpecAssertions.assertSlot;
import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.List;
import org.junit.jupiter.api.Test;

final class EconomyDynamicMenuSpecTest {
    @Test
    void voteListUsesDaemonDataAndSelectedLinkCommands() {
        var spec = VoteDynamicMenus.votes(List.of(new VoteMenuEntry("site", "vote.site", "https://example.test")));
        assertSlot(spec, 19, "vote.site");
        assertEquals(new MenuAction.RunPlayerCommand("vote site"), actionAt(spec, 19));
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
    void shopListEnablesItemsWithDeliveryMetadata() {
        var spec = ShopDynamicMenus.shop(List.of(new ShopMenuEntry("apple", "shop.apple", 5, true)));
        assertSlot(spec, 19, "shop.apple");
        assertEquals(new MenuAction.RunPlayerCommand("buy apple"), actionAt(spec, 19));
    }
}
