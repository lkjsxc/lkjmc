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
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("random-teleport-confirm"))), actionAt(spec, 22));
        assertEquals(new MenuAction.RunPlayerCommand("tpaccept"), actionAt(spec, 24));
    }

    @Test
    void randomTeleportQuoteControlsConfirmation() {
        var quote = new RandomTeleportQuote(true, true, 250, 300, 0, 750, 5000, 64);
        var spec = RandomTeleportDynamicMenus.confirm(quote);
        assertEquals(new MenuAction.RunPlayerCommand("rtp confirm"), actionAt(spec, 11));
        var cooldown = RandomTeleportDynamicMenus.confirm(new RandomTeleportQuote(true, true, 250, 300, 10, 750, 5000, 64));
        assertEquals(new MenuAction.Disabled("menu.random-teleport.disabled.cooldown"), actionAt(cooldown, 11));
    }

    @Test
    void homesMenuOffersFriendlyCreateFlow() {
        var spec = TravelDynamicMenus.homes(List.of());
        assertSlot(spec, 45, "menu.homes.set");
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("home-create-confirm"),
            java.util.Map.of("home", "home"))), actionAt(spec, 45));
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
        assertEquals(wake("alpha"), actionAt(spec, 19));
    }

    @Test
    void dynamicServerListAllowsSafeLifecycleCommands() {
        var permissions = new ServerMenuPermissions(true, true);
        var spec = DynamicMenus.serverList(List.of(
            new ServerMenuEntry("alpha", "purpur", "suspended", "process-absent", false, 0),
            new ServerMenuEntry("beta", "folia", "running", "process-healthy", true, 0),
            new ServerMenuEntry("gamma", "folia", "running", "process-healthy", true, 2)
        ), permissions);
        assertEquals(wake("alpha"), actionAt(spec, 19));
        assertEquals(new MenuAction.RunPlayerCommand("lkjmc server stop beta"), actionAt(spec, 20));
        assertEquals(new MenuAction.Disabled("menu.disabled.server-occupied"), actionAt(spec, 21));
    }

    @Test
    void adminServersListOpensSelectedServerDetail() {
        var permissions = adminPermissions();
        var spec = AdminServerDynamicMenus.servers(List.of(new ServerMenuEntry(
            "alpha", "paper", "stopped", "process-absent", false, 0)), permissions);
        assertSlot(spec, 19, "literal:alpha · stopped");
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("admin-server-detail"),
            java.util.Map.of("id", "alpha"))), actionAt(spec, 19));
    }

    @Test
    void adminServerCreateFlowChoosesKindTemplateThenGeneratesId() {
        var confirm = AdminServerDynamicMenus.createConfirm("folia", "folia-survival", adminPermissions());
        var payload = new MenuActionPayload(java.util.Map.of("id", "folia-survival-001", "kind", "folia",
            "template", "folia-survival", "acceptMinecraftEula", "true"));
        assertEquals(new MenuAction.DaemonCommand("instance.create", payload), actionAt(confirm, 22));
    }

    private static AdminMenuPermissions adminPermissions() {
        return new AdminMenuPermissions(true, true, true, true, true, true, true, true, true,
            true, true, true, true, true, true, true);
    }

    private static MenuAction wake(String id) {
        return new MenuAction.DaemonCommand("instance.wake.request", new MenuActionPayload("targetInstanceId=" + id));
    }
}
