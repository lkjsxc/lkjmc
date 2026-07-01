package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

public final class AchievementDynamicMenus {
    private static final List<Integer> CATEGORY = List.of(10, 11, 12, 13, 14, 15, 16);
    private static final List<Integer> ENTRY = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private AchievementDynamicMenus() {}

    public static MenuSpec loading() { return achievements(null); }
    public static MenuSpec achievements(List<AchievementMenuEntry> entries) { return achievements(entries, "all"); }

    public static MenuSpec achievements(List<AchievementMenuEntry> entries, String category) {
        var values = entries == null ? List.<AchievementMenuEntry>of() : List.copyOf(entries);
        var selected = category == null || category.isBlank() ? "all" : category;
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "DIAMOND", "menu.achievements.info", MenuAction.none(), ItemVisualRole.INFO,
            "literal:claimable=" + count(values, "claimable") + " in-progress=" + count(values, "in-progress")
                + " hidden=" + values.stream().filter(AchievementMenuEntry::hidden).count()));
        categories(slots, values, selected);
        var rows = values.stream().filter(entry -> visible(entry, selected)).sorted(order()).toList();
        for (int index = 0; index < rows.size() && index < ENTRY.size(); index++) {
            slots.put(ENTRY.get(index), row(ENTRY.get(index), rows.get(index)));
        }
        if (rows.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.achievements.empty", empty(),
                ItemVisualRole.DISABLED, "menu.achievements.empty.lore"));
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        addBorder(slots);
        return new MenuSpec(new MenuId("achievements"), new MenuTitle("menu.achievements.title"),
            new MenuSize(54), new ArrayList<>(slots.values()));
    }

    public static String progressBar(long current, long required) {
        var filled = (int) Math.min(10, Math.max(0, current) * 10 / Math.max(1, required));
        return "[" + "#".repeat(filled) + "-".repeat(10 - filled) + "]";
    }

    private static Comparator<AchievementMenuEntry> order() {
        return Comparator.comparingInt(AchievementDynamicMenus::rank).thenComparing(AchievementMenuEntry::id);
    }

    private static int rank(AchievementMenuEntry entry) {
        return switch (entry.state()) {
            case "claimable" -> 0;
            case "in-progress" -> 1;
            case "claimed" -> 2;
            default -> entry.hidden() ? 4 : 3;
        };
    }

    private static void categories(TreeMap<Integer, SlotSpec> slots, List<AchievementMenuEntry> entries, String selected) {
        slots.put(CATEGORY.get(0), category(CATEGORY.get(0), "all", selected));
        var cats = entries.stream().map(AchievementMenuEntry::category).distinct().sorted().limit(CATEGORY.size() - 1).toList();
        for (int index = 0; index < cats.size(); index++) { slots.put(CATEGORY.get(index + 1), category(CATEGORY.get(index + 1), cats.get(index), selected)); }
    }

    private static SlotSpec category(int slot, String category, String selected) {
        var active = category.equals(selected);
        var action = active ? MenuAction.none() : new MenuAction.OpenRoute(new MenuRoute(new MenuId("achievements"), Map.of("category", category)));
        return slot(slot, active ? "LIME_DYE" : "BOOK", "literal:" + category, action,
            active ? ItemVisualRole.INFO : ItemVisualRole.NAVIGATION);
    }

    private static boolean visible(AchievementMenuEntry entry, String category) {
        return (category.equals("all") || category.equals(entry.category())) && (!entry.hidden() || !entry.state().equals("locked"));
    }

    private static SlotSpec row(int slot, AchievementMenuEntry entry) {
        var title = entry.hidden() ? "menu.achievements.hidden" : entry.titleKey();
        var lore = List.of(entry.descriptionKey(), "literal:" + entry.category(),
            "literal:" + progressBar(entry.current(), entry.required()) + " " + entry.current() + "/" + entry.required(),
            "literal:" + entry.state(), "literal:" + entry.rewardSummary());
        if (entry.claimable()) {
            return new SlotSpec(slot, new ItemSpec(entry.iconMaterial(), title, add(lore, "menu.achievements.claim.lore"), ItemVisualRole.ACTION),
                new MenuAction.DaemonCommand("player.achievement.claim", new MenuActionPayload(Map.of("achievementId", entry.id()))));
        }
        var action = entry.disabledReason().isBlank() ? MenuAction.none() : new MenuAction.Disabled(entry.disabledReason());
        return new SlotSpec(slot, new ItemSpec(entry.iconMaterial(), title, lore, ItemVisualRole.INFO), action);
    }

    private static List<String> add(List<String> lore, String extra) { var copy = new ArrayList<>(lore); copy.add(extra); return copy; }
    private static long count(List<AchievementMenuEntry> entries, String state) { return entries.stream().filter(e -> e.state().equals(state)).count(); }
    private static MenuAction empty() { return new MenuAction.Disabled("menu.disabled.no-achievements"); }
    private static SlotSpec slot(int slot, String material, String key, MenuAction action, ItemVisualRole role, String... lore) { return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action); }
    private static void addBorder(TreeMap<Integer, SlotSpec> slots) { for (int border : borderSlots()) { slots.putIfAbsent(border, slot(border, MenuTheme.PROFILE.borderMaterial(), "menu.decorative", MenuAction.none(), ItemVisualRole.DECORATION)); } }
    private static List<Integer> borderSlots() { var slots = new ArrayList<Integer>(); for (int i = 0; i <= 8; i++) { slots.add(i); } for (int i = 45; i <= 53; i++) { slots.add(i); } slots.addAll(List.of(9, 18, 27, 36, 17, 26, 35, 44)); return slots; }
}
