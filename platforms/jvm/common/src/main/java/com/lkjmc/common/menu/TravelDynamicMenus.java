package com.lkjmc.common.menu;

import com.lkjmc.common.player.HomeNamePolicy;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.TreeMap;

public final class TravelDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private TravelDynamicMenus() {}

    public static MenuSpec homes(List<TravelMenuEntry> entries) {
        return menu("homes", "menu.homes.title", entries, "home", "RED_BED", "menu.homes.empty",
            "menu.homes.empty.lore", "travel");
    }

    public static MenuSpec warps(List<TravelMenuEntry> entries) {
        return menu("warps", "menu.warps.title", entries, "warp", "OAK_SIGN", "menu.warps.empty",
            "menu.warps.empty.lore", "travel");
    }

    private static MenuSpec menu(String id, String title, List<TravelMenuEntry> entries,
                                 String command, String material, String emptyKey,
                                 String emptyLore, String back) {
        var slots = new TreeMap<Integer, SlotSpec>();
        var sorted = entries == null ? List.<TravelMenuEntry>of() : entries.stream()
            .sorted(Comparator.comparing(TravelMenuEntry::name)).toList();
        for (int index = 0; index < sorted.size() && index < ENTRY_SLOTS.size(); index++) {
            var entry = sorted.get(index);
            slots.put(ENTRY_SLOTS.get(index), entrySlot(ENTRY_SLOTS.get(index), entry, command, material));
        }
        if (sorted.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", emptyKey, disabled(), ItemVisualRole.DISABLED, emptyLore));
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.TRAVEL.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(new MenuId(id), new MenuTitle(title), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec entrySlot(int slot, TravelMenuEntry entry, String command, String material) {
        if (command.equals("home") && !HomeNamePolicy.isValid(entry.name())) {
            return slot(slot, "BARRIER", "literal:" + entry.name(),
                new MenuAction.Disabled("menu.disabled.invalid-home-name"), ItemVisualRole.DISABLED,
                "literal:" + entry.serverId(), "menu.disabled.invalid-home-name");
        }
        return slot(slot, material, "literal:" + entry.name(),
            new MenuAction.RunPlayerCommand(command + " " + entry.name()), ItemVisualRole.ACTION,
            "literal:" + entry.serverId(), "menu.travel.teleport.lore");
    }

    private static MenuAction disabled() {
        return new MenuAction.Disabled("menu.disabled.no-travel-entries");
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
