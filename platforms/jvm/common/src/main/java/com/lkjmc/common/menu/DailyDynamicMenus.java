package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class DailyDynamicMenus {
    private DailyDynamicMenus() {}

    public static MenuSpec loading() {
        return daily(DailyRewardStatus.loading());
    }

    public static MenuSpec daily(DailyRewardStatus status) {
        var current = status == null ? DailyRewardStatus.loading() : status;
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "SUNFLOWER", "menu.daily.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.daily.info.lore"));
        slots.put(22, rewardSlot(current));
        slots.put(49, open(49, "ARROW", "menu.back", "economy", "menu.back.lore"));
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.ECONOMY.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(new MenuId("daily"), new MenuTitle("menu.daily.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec rewardSlot(DailyRewardStatus status) {
        if (!status.loaded()) {
            return slot(22, "CLOCK", "menu.daily.loading", new MenuAction.Disabled("menu.disabled.daily-loading"),
                ItemVisualRole.DISABLED, "menu.daily.loading.lore");
        }
        if (status.claimedToday()) {
            return slot(22, "GRAY_DYE", "menu.daily.claimed-today", new MenuAction.Disabled("menu.disabled.daily-claimed"),
                ItemVisualRole.DISABLED, "menu.daily.claimed-today.lore");
        }
        return slot(22, "SUNFLOWER", "menu.daily.claim", new MenuAction.RunPlayerCommand("daily"),
            ItemVisualRole.ACTION, "literal:+" + status.points() + " points", "menu.daily.claim.lore");
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
