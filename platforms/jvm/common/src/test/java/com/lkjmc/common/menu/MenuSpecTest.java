package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.List;

import com.google.gson.Gson;
import com.google.gson.reflect.TypeToken;
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
        var slot = new SlotSpec(4, item, new MenuAction.Command("menu"));
        var spec = new MenuSpec(new MenuId("root"), new MenuTitle("menu.root.title"), new MenuSize(54), List.of(slot));
        var decision = MenuReducer.click(spec, new MenuState(new MenuId("root"), 0), new MenuClick(4, "command:menu", true));
        assertEquals(new MenuEffect.RunCommand("menu"), decision.effects().get(0));
    }

    @Test
    void inertAndEmptyClicksAreSilent() {
        var spec = StandardMenus.root();
        assertTrue(MenuReducer.click(spec, new MenuState(spec.id(), 0), new MenuClick(0)).effects().isEmpty());
        assertTrue(MenuReducer.click(spec, new MenuState(spec.id(), 0), new MenuClick(30)).effects().isEmpty());
    }

    @Test
    void unknownMetadataReturnsFrameworkError() {
        var spec = StandardMenus.root();
        var decision = MenuReducer.click(spec, new MenuState(spec.id(), 0), new MenuClick(19, "bad", true));
        assertEquals(new MenuEffect.SendMessage("menu.error.unknown-action"), decision.effects().get(0));
        var stale = MenuReducer.click(spec, new MenuState(spec.id(), 0), new MenuClick(30, "command:stale", true));
        assertEquals(new MenuEffect.SendMessage("menu.error.unknown-action"), stale.effects().get(0));
    }

    @Test
    void standardMenusUseStableSlots() {
        assertSlot(StandardMenus.root(), 4, "menu.root.info");
        assertSlot(StandardMenus.root(), 19, "menu.network.title");
        assertSlot(StandardMenus.root(), 50, "menu.close");
        assertSlot(StandardMenus.language(), 20, "language.english");
        assertSlot(StandardMenus.language(), 24, "language.japanese");
        assertEquals(46, StandardMenus.navigation().previousSlot());
    }

    @Test
    void registryContainsRequiredMenus() {
        var registry = StandardMenus.registry();
        for (var id : List.of("root", "network", "homes", "warps", "teleports", "claims", "shop", "settings", "language")) {
            assertTrue(registry.find(new MenuId(id)).isPresent(), id);
        }
    }

    @Test
    void confirmationMenuHasConfirmAndCancel() {
        var spec = StandardMenus.confirmation(new ConfirmationSpec(new MenuId("confirm-delete"), "server.delete.confirm", new MenuAction.Command("confirm")));
        assertEquals(11, spec.slots().get(0).slot());
        assertEquals(15, spec.slots().get(1).slot());
    }

    @Test
    void standardItemKeysExistInEnglishAndJapanese() throws Exception {
        var en = locale("en");
        var ja = locale("ja");
        for (var menu : StandardMenus.registry().menus().values()) {
            assertTrue(en.containsKey(menu.title().key()), menu.title().key());
            assertTrue(ja.containsKey(menu.title().key()), menu.title().key());
            for (var slot : menu.slots()) {
                assertTrue(en.containsKey(slot.item().nameKey()), slot.item().nameKey());
                assertTrue(ja.containsKey(slot.item().nameKey()), slot.item().nameKey());
                assertFalse(slot.item().role() == ItemVisualRole.ACTION && MenuAction.key(slot.action()).equals("inert"));
            }
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
