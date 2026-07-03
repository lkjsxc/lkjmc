package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.TreeMap;

public final class KitDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private KitDynamicMenus() {}

    public static MenuSpec kits(List<KitMenuEntry> entries) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "IRON_SWORD", "menu.kits.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.kits.info.lore"));
        var sorted = entries == null ? List.<KitMenuEntry>of() : entries.stream()
            .sorted(Comparator.comparing(KitMenuEntry::id)).toList();
        for (int index = 0; index < sorted.size() && index < ENTRY_SLOTS.size(); index++) {
            slots.put(ENTRY_SLOTS.get(index), kitSlot(ENTRY_SLOTS.get(index), sorted.get(index)));
        }
        if (sorted.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.kits.empty", disabled(),
                ItemVisualRole.DISABLED, "menu.kits.empty.lore"));
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        MenuChrome.applyBorder(slots, MenuTheme.ECONOMY);
        return new MenuSpec(new MenuId("kits"), new MenuTitle("menu.kits.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec kitSlot(int slot, KitMenuEntry entry) {
        return slot(slot, "IRON_SWORD", entry.titleKey(),
            new MenuAction.RunPlayerCommand("kit claim " + entry.id()), ItemVisualRole.ACTION,
            "literal:+" + entry.rewardPoints() + " points", "literal:" + entry.cooldownHours() + "h cooldown",
            "menu.kits.claim.lore");
    }

    private static MenuAction disabled() {
        return new MenuAction.Disabled("menu.disabled.no-kits");
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
