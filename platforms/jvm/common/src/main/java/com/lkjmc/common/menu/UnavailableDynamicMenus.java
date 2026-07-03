package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class UnavailableDynamicMenus {
    private UnavailableDynamicMenus() {}

    public static MenuSpec unavailable(MenuId id, String titleKey, MenuTheme theme, String backRoute) {
        return unavailable(id, titleKey, theme, backRoute, "daemon.http_failed");
    }

    public static MenuSpec unavailable(MenuId id, String titleKey, MenuTheme theme, String backRoute, String code) {
        var diagnostic = MenuDiagnostic.of(code);
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(22, slot(22, "BARRIER", diagnostic.nameKey(),
            new MenuAction.Disabled(diagnostic.nameKey()), ItemVisualRole.DISABLED,
            diagnostic.loreKey()));
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        MenuChrome.applyBorder(slots, theme);
        return new MenuSpec(id, new MenuTitle(titleKey), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }

}
