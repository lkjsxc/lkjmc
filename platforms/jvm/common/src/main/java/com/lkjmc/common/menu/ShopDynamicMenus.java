package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.TreeMap;

public final class ShopDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private ShopDynamicMenus() {}

    public static MenuSpec shop(List<ShopMenuEntry> entries) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "EMERALD", "menu.shop.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.shop.info.lore"));
        var sorted = entries == null ? List.<ShopMenuEntry>of() : entries.stream()
            .sorted(Comparator.comparing(ShopMenuEntry::id)).toList();
        for (int index = 0; index < sorted.size() && index < ENTRY_SLOTS.size(); index++) {
            slots.put(ENTRY_SLOTS.get(index), itemSlot(ENTRY_SLOTS.get(index), sorted.get(index)));
        }
        if (sorted.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.shop.empty", disabled(),
                ItemVisualRole.DISABLED, "menu.shop.empty.lore"));
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.ECONOMY.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(new MenuId("shop"), new MenuTitle("menu.shop.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec itemSlot(int slot, ShopMenuEntry entry) {
        if (entry.deliveryAvailable()) {
            return slot(slot, "CHEST", entry.titleKey(), new MenuAction.RunPlayerCommand("buy " + entry.id()),
                ItemVisualRole.ACTION, "literal:" + entry.pricePoints() + " points", "menu.shop.buy.lore");
        }
        return slot(slot, "CHEST", entry.titleKey(), new MenuAction.Disabled("menu.disabled.shop-delivery"),
            ItemVisualRole.DISABLED, "literal:" + entry.pricePoints() + " points", "menu.disabled.shop-delivery");
    }

    private static MenuAction disabled() {
        return new MenuAction.Disabled("menu.disabled.no-shop-items");
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
