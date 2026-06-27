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
        slots.put(24, slot(24, "LIME_DYE", "menu.teleports.accept",
            new MenuAction.RunPlayerCommand("tpaccept"), ItemVisualRole.ACTION,
            "menu.teleports.accept.lore"));
        slots.put(49, open(49, "ARROW", "menu.back", "travel", "menu.back.lore"));
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.TRAVEL.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
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

    private static List<Integer> borderSlots() {
        var slots = new ArrayList<Integer>();
        for (int i = 0; i <= 8; i++) { slots.add(i); }
        for (int i = 45; i <= 53; i++) { slots.add(i); }
        slots.addAll(List.of(9, 18, 27, 36, 17, 26, 35, 44));
        return slots;
    }
}
