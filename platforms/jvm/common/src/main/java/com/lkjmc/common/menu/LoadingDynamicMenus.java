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
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        MenuChrome.applyBorder(slots, theme);
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

}
