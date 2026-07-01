package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

public final class AchievementDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private AchievementDynamicMenus() {}

    public static MenuSpec loading() {
        return achievements(null);
    }

    public static MenuSpec achievements(List<AchievementMenuEntry> entries) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "DIAMOND", "menu.achievements.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.achievements.info.lore"));
        var values = entries == null ? List.<AchievementMenuEntry>of() : entries.stream()
            .sorted(Comparator.comparing(AchievementMenuEntry::id)).toList();
        for (int index = 0; index < values.size() && index < ENTRY_SLOTS.size(); index++) {
            slots.put(ENTRY_SLOTS.get(index), achievementSlot(ENTRY_SLOTS.get(index), values.get(index)));
        }
        if (values.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.achievements.empty", empty(),
                ItemVisualRole.DISABLED, "menu.achievements.empty.lore"));
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.PROFILE.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(new MenuId("achievements"), new MenuTitle("menu.achievements.title"),
            new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec achievementSlot(int slot, AchievementMenuEntry entry) {
        var lore = "literal:" + entry.id() + " " + entry.current() + "/" + entry.required();
        if (entry.claimable()) {
            return slot(slot, "EXPERIENCE_BOTTLE", entry.titleKey(),
                new MenuAction.DaemonCommand("player.achievement.claim",
                    new MenuActionPayload(Map.of("achievementId", entry.id()))),
                ItemVisualRole.ACTION, lore, "menu.achievements.claim.lore");
        }
        var material = entry.rewardClaimed() ? "EMERALD" : "DIAMOND";
        return slot(slot, material, entry.titleKey(), MenuAction.none(), ItemVisualRole.INFO, lore);
    }

    private static MenuAction empty() { return new MenuAction.Disabled("menu.disabled.no-achievements"); }
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
