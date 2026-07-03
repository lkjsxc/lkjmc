package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.Test;

final class MenuChromeTest {
    @Test
    void commonChromeFramesStandardMenus() {
        for (var spec : List.of(StandardMenus.root(), StandardMenus.settings(),
            ShopDynamicMenus.shop(List.of()), UnavailableDynamicMenus.unavailable(
                new MenuId("shop"), "menu.shop.title", MenuTheme.ECONOMY, "economy"))) {
            for (var slot : MenuChrome.borderSlots()) {
                var item = MenuSpecAssertions.slotAt(spec, slot).item();
                assertTrue(item.role() == ItemVisualRole.DECORATION || item.role() == ItemVisualRole.NAVIGATION
                    || item.role() == ItemVisualRole.INFO,
                    spec.id() + ":" + slot);
            }
        }
    }

    @Test
    void economyDoesNotRenderClickOnlyBalanceRow() {
        var spec = StandardMenus.economy();
        assertTrue(spec.slots().stream().noneMatch(slot -> slot.item().nameKey().equals("menu.points.title")));
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("shop"))),
            MenuSpecAssertions.actionAt(spec, 19));
    }
}
