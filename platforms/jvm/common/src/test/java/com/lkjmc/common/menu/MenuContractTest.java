package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.bindings.ClaimPayload;
import com.lkjmc.bindings.ClaimSnapshot;
import com.lkjmc.bindings.MenuPayload;
import com.lkjmc.bindings.MenuSnapshot;
import com.lkjmc.bindings.PermissionPayload;
import com.lkjmc.bindings.PermissionSnapshot;
import com.lkjmc.bindings.SettingsPayload;
import com.lkjmc.bindings.SettingsSnapshot;
import java.io.ByteArrayInputStream;
import java.nio.charset.StandardCharsets;
import java.time.Instant;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.Test;

final class MenuContractTest {
    @Test
    void loadsEverySourceOwnedRoute() {
        var bundle = MenuBundle.fromResource();
        assertEquals(62, bundle.routes().size());
        assertEquals("root", bundle.route("root").id());
    }

    @Test
    void rejectsGenericMutationBody() throws Exception {
        String source;
        try (var input = MenuContractTest.class.getResourceAsStream("/lkjmc-menu-bundle.json")) {
            source = new String(input.readAllBytes(), StandardCharsets.UTF_8);
        }
        String mutated = source.replaceFirst("\\\"operation\\\":", "\\\"body\\\":{},\\\"operation\\\":");
        assertThrows(IllegalArgumentException.class, () -> MenuBundle.load(
                new ByteArrayInputStream(mutated.getBytes(StandardCharsets.UTF_8))));
    }

    @Test
    void integratesFourTypedRevisionedDomains() {
        var player = UUID.fromString("00000000-0000-0000-0000-000000000007");
        var now = Instant.parse("2026-07-14T00:00:00Z");
        var menu = new MenuSnapshot("menus", "global", 3, now, 2,
                new MenuPayload(List.of(), List.of(), List.of(), List.of()));
        var permission = new PermissionSnapshot("permissions", player.toString(), 4, now, 2,
                new PermissionPayload("player", player.toString(), List.of(), List.of("menu.action.home-save")));
        var claim = new ClaimSnapshot("claims", player.toString(), 5, now, 2,
                new ClaimPayload(List.of()));
        var settings = new SettingsSnapshot("settings", player.toString(), 6, now, 2,
                new SettingsPayload(player, "ja", true, true, true, Map.of()));
        var view = MenuSnapshotView.of(MenuTypes.Freshness.CURRENT, menu, permission, claim, settings);
        assertEquals(3, view.entry(MenuTypes.Domain.MENUS).revision());
        assertEquals(5, view.entry(MenuTypes.Domain.CLAIMS).revision());
        assertEquals(6, view.entry(MenuTypes.Domain.SETTINGS).revision());
        assertTrue(view.hasCurrentCapability("menu.action.home-save"));
    }

    @Test
    void stalePermissionNeverAuthorizes() {
        var player = UUID.fromString("00000000-0000-0000-0000-000000000008");
        var permission = new PermissionSnapshot("permissions", player.toString(), 2, Instant.EPOCH, 1,
                new PermissionPayload("player", player.toString(), List.of(), List.of("menu.action.home-save")));
        assertTrue(!MenuSnapshotView.of(MenuTypes.Freshness.STALE, permission)
                .hasCurrentCapability("menu.action.home-save"));
    }
}
