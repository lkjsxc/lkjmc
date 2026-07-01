package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

public final class ShopDynamicMenus {
    private static final List<Integer> CATEGORY_SLOTS = List.of(10, 11, 12, 13, 14, 15, 16);
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private ShopDynamicMenus() {}

    public static MenuSpec shop(List<ShopMenuEntry> entries) {
        return shop(new ShopView(0, "all", entries));
    }

    public static MenuSpec shop(ShopView view) {
        var current = view == null ? new ShopView(0, "all", List.of()) : view;
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "EMERALD", "menu.shop.info", MenuAction.none(),
            ItemVisualRole.INFO, "literal:Balance: " + current.balance()));
        categories(slots, current);
        var sorted = current.entries().stream().filter(entry -> selected(current, entry))
            .sorted(Comparator.comparing(ShopMenuEntry::category).thenComparing(ShopMenuEntry::id)).toList();
        for (int index = 0; index < sorted.size() && index < ENTRY_SLOTS.size(); index++) {
            slots.put(ENTRY_SLOTS.get(index), itemSlot(ENTRY_SLOTS.get(index), current.balance(), sorted.get(index)));
        }
        if (sorted.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.shop.empty", disabled("menu.disabled.no-shop-items"),
                ItemVisualRole.DISABLED, "menu.shop.empty.lore"));
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        addBorder(slots);
        return new MenuSpec(new MenuId("shop"), new MenuTitle("menu.shop.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static void categories(TreeMap<Integer, SlotSpec> slots, ShopView view) {
        var categories = view.entries().stream().map(ShopMenuEntry::category).distinct().sorted().limit(CATEGORY_SLOTS.size() - 1).toList();
        slots.put(CATEGORY_SLOTS.get(0), categorySlot(CATEGORY_SLOTS.get(0), "all", view.category()));
        for (int index = 0; index < categories.size(); index++) {
            slots.put(CATEGORY_SLOTS.get(index + 1), categorySlot(CATEGORY_SLOTS.get(index + 1), categories.get(index), view.category()));
        }
    }

    private static SlotSpec categorySlot(int slot, String category, String selected) {
        var selectedCategory = category.equals(selected);
        var action = selectedCategory ? MenuAction.none()
            : new MenuAction.OpenRoute(new MenuRoute(new MenuId("shop"), Map.of("category", category)));
        return slot(slot, selectedCategory ? "LIME_STAINED_GLASS_PANE" : "YELLOW_STAINED_GLASS_PANE",
            "literal:" + category, action, selectedCategory ? ItemVisualRole.INFO : ItemVisualRole.NAVIGATION,
            selectedCategory ? "literal:Selected category" : "literal:Filter this category");
    }

    private static boolean selected(ShopView view, ShopMenuEntry entry) {
        return view.category().equals("all") || view.category().equals(entry.category());
    }

    private static SlotSpec itemSlot(int slot, long balance, ShopMenuEntry entry) {
        var lore = List.of("literal:" + entry.category(), "literal:" + entry.material() + " x" + entry.amount(),
            "literal:Price: " + entry.pricePoints() + " · Balance: " + balance, "literal:Delivery: " + entry.deliveryKind());
        if (!entry.deliveryAvailable()) {
            return item(slot, entry, disabled(reason(entry, "menu.disabled.shop-delivery")), ItemVisualRole.DISABLED, lore);
        }
        if (!entry.affordable()) {
            return item(slot, entry, disabled(reason(entry, "menu.disabled.shop-afford")), ItemVisualRole.DISABLED, lore);
        }
        return item(slot, entry, new MenuAction.RunPlayerCommand("buy " + entry.id()), ItemVisualRole.ACTION,
            add(lore, "menu.shop.buy.lore"));
    }

    private static SlotSpec item(int slot, ShopMenuEntry entry, MenuAction action, ItemVisualRole role, List<String> lore) {
        return new SlotSpec(slot, new ItemSpec(entry.material(), entry.titleKey(), lore, role), action);
    }

    private static String reason(ShopMenuEntry entry, String fallback) {
        return entry.disabledReason().isBlank() ? fallback : entry.disabledReason();
    }

    private static List<String> add(List<String> lore, String extra) {
        var copy = new ArrayList<>(lore);
        copy.add(extra);
        return copy;
    }

    private static MenuAction disabled(String reason) { return new MenuAction.Disabled(reason); }
    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }
    private static void addBorder(TreeMap<Integer, SlotSpec> slots) {
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.ECONOMY.borderMaterial(), "menu.decorative",
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
