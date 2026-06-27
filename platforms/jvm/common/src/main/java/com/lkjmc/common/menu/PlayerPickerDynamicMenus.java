package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.TreeMap;

public final class PlayerPickerDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private PlayerPickerDynamicMenus() {}

    public static MenuSpec picker(String id, String titleKey, MenuTheme theme, String back,
                                  String commandPrefix, List<PlayerMenuEntry> players) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "PLAYER_HEAD", "menu.player-picker.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.player-picker.info.lore"));
        var sorted = players == null ? List.<PlayerMenuEntry>of() : players.stream()
            .sorted(Comparator.comparing(PlayerMenuEntry::name)).toList();
        for (int index = 0; index < sorted.size() && index < ENTRY_SLOTS.size(); index++) {
            slots.put(ENTRY_SLOTS.get(index), playerSlot(ENTRY_SLOTS.get(index), sorted.get(index), commandPrefix));
        }
        if (sorted.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.player-picker.empty", empty(),
                ItemVisualRole.DISABLED, "menu.player-picker.empty.lore"));
        }
        slots.put(49, MenuChrome.back());
        addBorder(slots, theme);
        return new MenuSpec(new MenuId(id), new MenuTitle(titleKey), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec playerSlot(int slot, PlayerMenuEntry entry, String commandPrefix) {
        return slot(slot, "PLAYER_HEAD", "literal:" + entry.name(),
            new MenuAction.RunPlayerCommand(commandPrefix + " " + entry.name()), ItemVisualRole.ACTION,
            "menu.player-picker.select.lore");
    }

    private static MenuAction empty() { return new MenuAction.Disabled("menu.disabled.no-pickable-players"); }
    private static SlotSpec open(int slot, String material, String key, String menu, String... lore) {
        return slot(slot, material, key, new MenuAction.OpenRoute(new MenuRoute(new MenuId(menu))),
            ItemVisualRole.NAVIGATION, lore);
    }
    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }
    private static void addBorder(TreeMap<Integer, SlotSpec> slots, MenuTheme theme) {
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, theme.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
    }
    private static List<Integer> borderSlots() {
        var slots = new ArrayList<Integer>();
        for (int i = 0; i <= 8; i++) { slots.add(i); }
        for (int i = 45; i <= 53; i++) { slots.add(i); }
        slots.addAll(List.of(9, 18, 27, 36, 17, 26, 35, 44));
        return slots;
    }
}
