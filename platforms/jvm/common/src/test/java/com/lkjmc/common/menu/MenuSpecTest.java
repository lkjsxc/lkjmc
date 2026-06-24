package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.List;

import org.junit.jupiter.api.Test;

final class MenuSpecTest {
    @Test
    void rejectsDuplicateSlots() {
        var item = new ItemSpec("STONE", "stone", List.of());
        var first = new SlotSpec(4, item, MenuAction.none());
        var second = new SlotSpec(4, item, MenuAction.none());
        assertThrows(IllegalArgumentException.class, () -> new MenuSpec(
            new MenuId("root"),
            new MenuTitle("menu.root.title"),
            new MenuSize(54),
            List.of(first, second)
        ));
    }

    @Test
    void clickProducesCommandEffect() {
        var item = new ItemSpec("COMPASS", "menu.root.title", List.of());
        var slot = new SlotSpec(4, item, new MenuAction.Command("menu"));
        var spec = new MenuSpec(new MenuId("root"), new MenuTitle("menu.root.title"), new MenuSize(54), List.of(slot));
        var decision = MenuReducer.click(spec, new MenuState(new MenuId("root"), 0), new MenuClick(4));
        assertEquals(1, decision.effects().size());
        assertEquals(new MenuEffect.RunCommand("menu"), decision.effects().get(0));
    }
}
