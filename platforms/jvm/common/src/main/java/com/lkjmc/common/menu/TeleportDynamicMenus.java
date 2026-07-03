package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class TeleportDynamicMenus {
    private TeleportDynamicMenus() {}

    public static MenuSpec teleports() {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "ENDER_PEARL", "menu.teleports.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.teleports.info.lore"));
        slots.put(20, open(20, "ENDER_PEARL", "menu.teleports.request", "teleport-picker",
            "menu.teleports.request.lore"));
        slots.put(22, open(22, "CHORUS_FRUIT", "menu.random-teleport.title", "random-teleport-overworld",
            "menu.random-teleport.lore"));
        slots.put(24, slot(24, "LIME_DYE", "menu.teleports.accept",
            new MenuAction.RunPlayerCommand("tpaccept"), ItemVisualRole.ACTION,
            "menu.teleports.accept.lore"));
        slots.put(49, MenuChrome.back());
        MenuChrome.applyBorder(slots, MenuTheme.TRAVEL);
        return new MenuSpec(new MenuId("teleports"), new MenuTitle("menu.teleports.title"),
            new MenuSize(54), new ArrayList<>(slots.values()));
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
