package com.lkjmc.common.menu;

import static com.lkjmc.common.menu.MenuSpecAssertions.actionAt;
import static com.lkjmc.common.menu.MenuSpecAssertions.assertSlot;
import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

final class TravelClaimDynamicMenuSpecTest {
    @Test
    void claimListUsesDaemonDataAndConfirmsDelete() {
        var spec = ClaimDynamicMenus.claims(List.of(new ClaimMenuEntry("base", 2)));
        assertEquals(new MenuAction.TextInput("menu.input.claim-name.prompt", "claim create"), actionAt(spec, 40));
        assertSlot(spec, 19, "literal:base");
        var route = new MenuRoute(new MenuId("claim-detail"), Map.of("name", "base", "chunkCount", "2"));
        assertEquals(new MenuAction.OpenRoute(route), actionAt(spec, 19));
        var detail = ClaimDynamicMenus.claimDetail("base", 2);
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("claim-confirm"), Map.of("name", "base"))),
            actionAt(detail, 20));
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("claim-trust-picker"), Map.of("name", "base"))),
            actionAt(detail, 24));
        assertEquals(new MenuAction.RunPlayerCommand("claim delete base"), actionAt(ClaimDynamicMenus.claimConfirm("base"), 11));
    }

    @Test
    void travelListsUseDaemonDataAndCommandPayloads() {
        var spec = TravelDynamicMenus.homes(List.of(
            new TravelMenuEntry("zeta", "hub"), new TravelMenuEntry("alpha", "survival")));
        assertSlot(spec, 19, "literal:alpha");
        assertEquals(new MenuAction.RunPlayerCommand("home alpha"), actionAt(spec, 19));
    }

    @Test
    void homesWithUnsafeCommandNamesAreDisabled() {
        var spec = TravelDynamicMenus.homes(List.of(new TravelMenuEntry("bad name", "hub")));
        assertSlot(spec, 19, "literal:bad name");
        assertEquals(new MenuAction.Disabled("menu.disabled.invalid-home-name"), actionAt(spec, 19));
    }
}
