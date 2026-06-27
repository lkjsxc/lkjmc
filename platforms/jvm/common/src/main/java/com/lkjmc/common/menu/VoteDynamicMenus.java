package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.TreeMap;

public final class VoteDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private VoteDynamicMenus() {}

    public static MenuSpec votes(List<VoteMenuEntry> entries) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "PAPER", "menu.votes.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.votes.info.lore"));
        var sorted = entries == null ? List.<VoteMenuEntry>of() : entries.stream()
            .sorted(Comparator.comparing(VoteMenuEntry::id)).toList();
        for (int index = 0; index < sorted.size() && index < ENTRY_SLOTS.size(); index++) {
            slots.put(ENTRY_SLOTS.get(index), voteSlot(ENTRY_SLOTS.get(index), sorted.get(index)));
        }
        if (sorted.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.votes.empty", disabled(),
                ItemVisualRole.DISABLED, "menu.votes.empty.lore"));
        }
        slots.put(49, open(49, "ARROW", "menu.back", "economy", "menu.back.lore"));
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.ECONOMY.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(new MenuId("votes"), new MenuTitle("menu.votes.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec voteSlot(int slot, VoteMenuEntry entry) {
        return slot(slot, "PAPER", entry.titleKey(), new MenuAction.Disabled("menu.disabled.vote-open"),
            ItemVisualRole.DISABLED, "literal:" + entry.url(), "menu.disabled.vote-open");
    }

    private static MenuAction disabled() {
        return new MenuAction.Disabled("menu.disabled.no-vote-links");
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
