package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.Gson;
import com.google.gson.reflect.TypeToken;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.List;
import org.junit.jupiter.api.Test;

final class MenuSpecTest {
    @Test
    void rejectsDuplicateSlots() {
        var item = new ItemSpec("STONE", "stone", List.of());
        var first = new SlotSpec(4, item, MenuAction.none());
        var second = new SlotSpec(4, item, MenuAction.none());
        assertThrows(IllegalArgumentException.class, () -> new MenuSpec(
            new MenuId("root"), new MenuTitle("menu.root.title"), new MenuSize(54), List.of(first, second)));
    }

    @Test
    void clickProducesCommandEffect() {
        var item = new ItemSpec("COMPASS", "menu.root.title", List.of());
        var slot = new SlotSpec(4, item, new MenuAction.RunPlayerCommand("menu"));
        var spec = new MenuSpec(new MenuId("root"), new MenuTitle("menu.root.title"), new MenuSize(54), List.of(slot));
        var decision = MenuReducer.click(spec, new MenuState(new MenuId("root"), 0), new MenuClick(4, "command:menu", true));
        assertEquals(new MenuEffect.RunPlayerCommand("menu"), decision.effects().get(0));
    }

    @Test
    void inertAndEmptyClicksAreSilent() {
        var spec = StandardMenus.root();
        assertTrue(MenuReducer.click(spec, new MenuState(spec.id(), 0), new MenuClick(0)).effects().isEmpty());
        assertTrue(MenuReducer.click(spec, new MenuState(spec.id(), 0), new MenuClick(30)).effects().isEmpty());
    }

    @Test
    void metadataClassificationsAreLocalizedFailures() {
        var spec = StandardMenus.root();
        var state = new MenuState(new MenuRoute(spec.id()), new MenuRouteStack(List.of(new MenuRoute(spec.id()))), 0, "s1", 2);
        assertFailure(spec, state, new MenuClick(19, "bad", true), MenuFailure.UNKNOWN_METADATA);
        assertFailure(spec, state, new MenuClick(30, "command:stale", true), MenuFailure.UNKNOWN_METADATA);
        var slot = spec.slots().stream().filter(value -> value.slot() == 19).findFirst().orElseThrow();
        assertFailure(spec, state, click(slot, "other", 2, spec.id()), MenuFailure.STALE_SESSION);
        assertFailure(spec, state, click(slot, "s1", 1, spec.id()), MenuFailure.STALE_EPOCH);
        assertFailure(spec, state, click(slot, "s1", 2, new MenuId("settings")), MenuFailure.ROUTE_MISMATCH);
    }

    @Test
    void disabledActionReturnsReason() {
        var spec = StandardMenus.root();
        var decision = MenuReducer.click(spec, new MenuState(spec.id(), 0), new MenuClick(31, "disabled:menu.disabled.staff", true));
        assertEquals(new MenuEffect.SendMessage("menu.disabled.staff"), decision.effects().get(0));
    }

    @Test
    void onlyCloseActionProducesCloseEffect() {
        for (var menu : StandardMenus.registry().menus().values()) {
            for (var slot : menu.slots()) {
                var hasClose = MenuReducer.click(menu, new MenuState(menu.id(), 0),
                    new MenuClick(slot.slot(), MenuAction.key(slot.action()), true)).effects()
                    .stream().anyMatch(MenuEffect.CloseMenu.class::isInstance);
                assertEquals(slot.action() instanceof MenuAction.Close, hasClose, menu.id() + ":" + slot.slot());
            }
        }
    }

    @Test
    void standardMenusUseStableSlots() {
        assertSlot(StandardMenus.root(), 4, "menu.root.info");
        assertSlot(StandardMenus.root(), 19, "menu.network.title");
        assertSlot(StandardMenus.root(), 50, "menu.close");
        assertSlot(StandardMenus.settings(), 24, "menu.hotbar-token.toggle");
        assertEquals("NETHER_STAR", StandardMenus.settings().slots().stream()
            .filter(value -> value.slot() == 24).findFirst().orElseThrow().item().material());
        assertSlot(StandardMenus.language(), 20, "language.english");
        assertSlot(StandardMenus.language(), 24, "language.japanese");
        assertEquals(46, StandardMenus.navigation().previousSlot());
    }

    @Test
    void registryContainsRequiredMenus() {
        var registry = StandardMenus.registry();
        for (var id : List.of("root", "network", "server-list", "server-detail", "travel", "homes", "warps",
            "teleports", "teleport-picker", "claims", "claim-detail", "claim-confirm", "claim-trust-picker",
            "economy", "shop", "shop-detail", "kits", "daily", "votes", "social", "party", "party-confirm",
            "party-invite-picker", "mail", "reports", "report-detail", "report-confirm", "profile", "achievements",
            "settings", "language", "adventures", "adventures-end-confirm", "adventures-end-party-confirm")) {
            assertTrue(registry.find(new MenuId(id)).isPresent(), id);
        }
    }

    @Test
    void confirmationMenuHasConfirmAndCancel() {
        var spec = StandardMenus.confirmation(new ConfirmationSpec(new MenuId("confirm-delete"), "server.delete.confirm", new MenuAction.RunPlayerCommand("confirm")));
        assertEquals(11, spec.slots().get(0).slot());
        assertEquals(15, spec.slots().get(1).slot());
    }

    @Test
    void paginationClampsBounds() {
        var pagination = new Pagination(4, 10, 12);
        assertEquals(1, pagination.clampedPage());
        assertFalse(pagination.hasNext());
        assertTrue(pagination.hasPrevious());
    }

    @Test
    void standardItemKeysExistInEnglishAndJapanese() throws Exception {
        var en = locale("en");
        var ja = locale("ja");
        for (var menu : StandardMenus.registry().menus().values()) {
            assertTrue(en.containsKey(menu.title().key()), menu.title().key());
            assertTrue(ja.containsKey(menu.title().key()), menu.title().key());
            for (var slot : menu.slots()) {
                assertLocaleKey(en, slot.item().nameKey());
                assertLocaleKey(ja, slot.item().nameKey());
                for (var lore : slot.item().loreKeys()) {
                    assertLocaleKey(en, lore);
                    assertLocaleKey(ja, lore);
                }
                assertFalse(slot.item().role() == ItemVisualRole.ACTION && MenuAction.key(slot.action()).equals("none"));
            }
        }
        for (var failure : MenuFailure.values()) {
            assertTrue(en.containsKey(failure.messageKey()), failure.messageKey());
            assertTrue(ja.containsKey(failure.messageKey()), failure.messageKey());
        }
        for (var code : List.of("daemon.not_configured", "daemon.token_missing", "daemon.token_unreadable",
            "daemon.http_failed", "daemon.auth_failed", "daemon.command_unknown", "daemon.command_failed",
            "database.not_configured", "database.unavailable", "menu.schema_mismatch", "menu.permission_denied")) {
            var diagnostic = MenuDiagnostic.of(code);
            assertTrue(en.containsKey(diagnostic.nameKey()), diagnostic.nameKey());
            assertTrue(en.containsKey(diagnostic.loreKey()), diagnostic.loreKey());
            assertTrue(ja.containsKey(diagnostic.nameKey()), diagnostic.nameKey());
            assertTrue(ja.containsKey(diagnostic.loreKey()), diagnostic.loreKey());
        }
    }

    private static void assertFailure(MenuSpec spec, MenuState state, MenuClick click, MenuFailure failure) {
        var decision = MenuReducer.click(spec, state, click);
        assertEquals(failure, decision.failure());
        assertEquals(new MenuEffect.SendMessage(failure.messageKey()), decision.effects().get(0));
    }

    private static MenuClick click(SlotSpec slot, String session, long epoch, MenuId menu) {
        var metadata = MenuMetadata.of(menu, new MenuRoute(menu), slot.slot(), slot.action(), session, epoch, slot.item().inert());
        return new MenuClick(slot.slot(), metadata, null, true);
    }

    private static void assertLocaleKey(java.util.Map<String, String> values, String key) {
        if (!key.startsWith("literal:")) {
            assertTrue(values.containsKey(key), key);
        }
    }

    private static void assertSlot(MenuSpec spec, int slot, String key) {
        assertEquals(key, spec.slots().stream().filter(value -> value.slot() == slot).findFirst().orElseThrow().item().nameKey());
    }

    private static java.util.Map<String, String> locale(String id) throws Exception {
        var stream = MenuSpecTest.class.getClassLoader().getResourceAsStream("locales/" + id + ".json");
        return new Gson().fromJson(new InputStreamReader(stream, StandardCharsets.UTF_8), new TypeToken<java.util.Map<String, String>>() {}.getType());
    }
}
