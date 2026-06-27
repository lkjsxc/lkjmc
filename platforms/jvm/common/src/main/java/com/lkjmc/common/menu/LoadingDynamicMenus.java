package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class LoadingDynamicMenus {
    private LoadingDynamicMenus() {}

    public static MenuSpec loading(MenuId id, String titleKey, MenuTheme theme, String backRoute) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(22, slot(22, "CLOCK", "menu.loading.live-data",
            new MenuAction.Disabled("menu.disabled.dynamic-loading"), ItemVisualRole.DISABLED,
            "menu.loading.live-data.lore"));
        slots.put(49, open(49, "ARROW", "menu.back", backRoute, "menu.back.lore"));
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, theme.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(id, new MenuTitle(titleKey), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec open(int slot, String material, String key, String menu, String... lore) {
        return slot(slot, material, key, new MenuAction.OpenRoute(new MenuRoute(new MenuId(menu))),
            ItemVisualRole.NAVIGATION, lore);
    }

    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }

    private static List<Integer> borderSlots() {
        var slots = new ArrayList<Integer>();
        for (int i = 0; i <= 8; i++) { slots.add(i); }
        for (int i = 45; i <= 53; i++) { slots.add(i); }
        slots.addAll(List.of(9, 18, 27, 36, 17, 26, 35, 44));
        return slots;
    }
}
