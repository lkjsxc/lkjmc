package com.lkjmc.common.menu;

import static com.lkjmc.common.menu.MenuSpecAssertions.actionAt;
import static com.lkjmc.common.menu.MenuSpecAssertions.assertSlot;
import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.List;
import org.junit.jupiter.api.Test;

final class DynamicMenuSpecTest {
    @Test
    void loadingMenuBlocksEarlyClicks() {
        var spec = LoadingDynamicMenus.loading(new MenuId("homes"), "menu.homes.title", MenuTheme.TRAVEL, "travel");
        assertSlot(spec, 22, "menu.loading.live-data");
        assertEquals(new MenuAction.Disabled("menu.disabled.dynamic-loading"), actionAt(spec, 22));
    }

    @Test
    void unavailableMenuExplainsTypedDiagnostics() {
        var spec = UnavailableDynamicMenus.unavailable(new MenuId("shop"), "menu.shop.title",
            MenuTheme.ECONOMY, "economy", "daemon.token_missing");
        assertSlot(spec, 22, "menu.unavailable.daemon.token-missing");
        assertEquals(new MenuAction.Disabled("menu.unavailable.daemon.token-missing"), actionAt(spec, 22));
    }

    @Test
    void teleportMenuUsesPlayerPickerForRequests() {
        var spec = TeleportDynamicMenus.teleports();
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("teleport-picker"))), actionAt(spec, 20));
        assertEquals(new MenuAction.RunPlayerCommand("tpaccept"), actionAt(spec, 24));
    }

    @Test
    void playerPickerRunsTargetedCommands() {
        var spec = PlayerPickerDynamicMenus.picker("teleport-picker", "menu.teleports.picker.title", MenuTheme.TRAVEL,
            "teleports", "tpa", List.of(new PlayerMenuEntry("Alex")));
        assertSlot(spec, 19, "literal:Alex");
        assertEquals(new MenuAction.RunPlayerCommand("tpa Alex"), actionAt(spec, 19));
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
}
