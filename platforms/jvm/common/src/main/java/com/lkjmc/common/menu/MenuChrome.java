package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;

public final class MenuChrome {
    private MenuChrome() {}

    public static SlotSpec back() {
        return slot(49, "ARROW", "menu.back", new MenuAction.Back(),
            ItemVisualRole.NAVIGATION, "menu.back.lore");
    }

    public static SlotSpec refresh() {
        return slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore");
    }

    public static SlotSpec close() {
        return slot(50, "BARRIER", "menu.close", new MenuAction.Close(),
            ItemVisualRole.NAVIGATION);
    }

    public static SlotSpec decoration(int slot, MenuTheme theme) {
        return slot(slot, theme.borderMaterial(), "menu.decorative",
            MenuAction.none(), ItemVisualRole.DECORATION);
    }

    public static List<Integer> borderSlots() {
        var slots = new ArrayList<Integer>();
        for (int i = 0; i <= 8; i++) { slots.add(i); }
        for (int i = 45; i <= 53; i++) { slots.add(i); }
        slots.addAll(List.of(9, 18, 27, 36, 17, 26, 35, 44));
        return slots;
    }

    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }
}
