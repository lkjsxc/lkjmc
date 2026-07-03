package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

public final class AchievementDynamicMenus {
    private static final List<Integer> ENTRY = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private AchievementDynamicMenus() {}

    public static MenuSpec loading() { return achievements(null); }
    public static MenuSpec achievements(List<AchievementMenuEntry> entries) { return root(entries); }
    public static MenuSpec achievements(List<AchievementMenuEntry> entries, String ignored) { return root(entries); }

    public static MenuSpec root(List<AchievementMenuEntry> entries) {
        var values = visible(entries);
        var slots = base("achievements", "menu.achievements.title");
        slots.put(4, info(values));
        var directories = directories(values);
        for (int i = 0; i < directories.size() && i < ENTRY.size(); i++) {
            slots.put(ENTRY.get(i), directory(ENTRY.get(i), directories.get(i)));
        }
        if (directories.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.achievements.empty", empty(),
                ItemVisualRole.DISABLED, "menu.achievements.empty.lore"));
        }
        slots.put(45, MenuChrome.mainMenu(45));
        slots.put(49, MenuChrome.back());
        slots.put(50, MenuChrome.refresh());
        return menu("achievements", "menu.achievements.title", slots);
    }

    public static MenuSpec directory(List<AchievementMenuEntry> entries, String path) {
        var values = visible(entries).stream()
            .filter(entry -> path.equals("claimable") ? entry.claimable() : category(entry).equals(path))
            .sorted(order()).toList();
        var slots = base("achievement-directory", "menu.achievements.directory.title");
        slots.put(4, slot(4, "BOOK", "literal:" + path, MenuAction.none(), ItemVisualRole.INFO,
            "literal:" + values.size() + " achievements"));
        for (int i = 0; i < values.size() && i < ENTRY.size(); i++) {
            slots.put(ENTRY.get(i), row(ENTRY.get(i), values.get(i)));
        }
        if (values.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.achievements.empty", empty(),
                ItemVisualRole.DISABLED, "menu.achievements.empty.lore"));
        }
        slots.put(45, MenuChrome.mainMenu(45));
        slots.put(49, MenuChrome.parentDirectory());
        slots.put(50, MenuChrome.refresh());
        return menu("achievement-directory", "menu.achievements.directory.title", slots);
    }

    public static MenuSpec detail(List<AchievementMenuEntry> entries, String id) {
        var entry = visible(entries).stream().filter(value -> value.id().equals(id)).findFirst();
        if (entry.isEmpty()) {
            var slots = base("achievement-detail", "menu.achievements.detail.title");
            slots.put(22, slot(22, "BARRIER", "menu.achievements.empty", empty(), ItemVisualRole.DISABLED,
                "menu.achievements.empty.lore"));
            slots.put(45, MenuChrome.mainMenu(45));
            slots.put(49, MenuChrome.parentDirectory());
            return menu("achievement-detail", "menu.achievements.detail.title", slots);
        }
        var value = entry.get();
        var slots = base("achievement-detail", "menu.achievements.detail.title");
        var lore = List.of(value.descriptionKey(), "literal:" + category(value),
            "literal:" + progressBar(value.current(), value.required()) + " " + value.current() + "/" + value.required(),
            "literal:" + value.state(), "literal:" + value.rewardSummary());
        slots.put(22, new SlotSpec(22, new ItemSpec(value.iconMaterial(), value.titleKey(), lore, ItemVisualRole.INFO), MenuAction.none()));
        slots.put(31, claim(value));
        slots.put(45, MenuChrome.mainMenu(45));
        slots.put(49, MenuChrome.parentDirectory());
        slots.put(50, MenuChrome.refresh());
        return menu("achievement-detail", "menu.achievements.detail.title", slots);
    }

    public static String progressBar(long current, long required) {
        var filled = (int) Math.min(10, Math.max(0, current) * 10 / Math.max(1, required));
        return "[" + "#".repeat(filled) + "-".repeat(10 - filled) + "]";
    }

    private static TreeMap<Integer, SlotSpec> base(String id, String title) { return new TreeMap<>(); }
    private static SlotSpec info(List<AchievementMenuEntry> values) {
        return slot(4, "DIAMOND", "menu.achievements.info", MenuAction.none(), ItemVisualRole.INFO,
            "literal:claimable=" + count(values, "claimable") + " in-progress=" + count(values, "in-progress"));
    }
    private static List<String> directories(List<AchievementMenuEntry> entries) {
        var dirs = new ArrayList<String>();
        if (entries.stream().anyMatch(AchievementMenuEntry::claimable)) { dirs.add("claimable"); }
        entries.stream().map(AchievementDynamicMenus::category).distinct().sorted().forEach(dirs::add);
        return List.copyOf(dirs);
    }
    private static SlotSpec directory(int slot, String path) {
        return slot(slot, path.equals("claimable") ? "EMERALD" : "BOOK", "literal:" + path,
            new MenuAction.OpenRoute(new MenuRoute(new MenuId("achievement-directory"), Map.of("path", path))),
            ItemVisualRole.NAVIGATION, "menu.achievements.directory.lore");
    }
    private static SlotSpec row(int slot, AchievementMenuEntry entry) {
        return slot(slot, entry.iconMaterial(), entry.titleKey(),
            new MenuAction.OpenRoute(new MenuRoute(new MenuId("achievement-detail"), Map.of("id", entry.id()))),
            ItemVisualRole.NAVIGATION, entry.descriptionKey(), "literal:" + entry.state());
    }
    private static SlotSpec claim(AchievementMenuEntry entry) {
        if (entry.claimable()) {
            return slot(31, "EMERALD", "menu.achievements.claim",
                new MenuAction.DaemonCommand("player.achievement.claim", MenuActionPayload.of("achievementId", entry.id())),
                ItemVisualRole.ACTION, "menu.achievements.claim.lore");
        }
        return slot(31, "BARRIER", "menu.achievements.disabled.not-claimable",
            new MenuAction.Disabled(reason(entry)), ItemVisualRole.DISABLED, reason(entry));
    }
    private static List<AchievementMenuEntry> visible(List<AchievementMenuEntry> entries) {
        return (entries == null ? List.<AchievementMenuEntry>of() : entries).stream()
            .filter(entry -> !entry.hidden() || !entry.state().equals("locked")).toList();
    }
    private static Comparator<AchievementMenuEntry> order() {
        return Comparator.comparingInt(AchievementDynamicMenus::rank).thenComparing(AchievementMenuEntry::id);
    }
    private static int rank(AchievementMenuEntry entry) {
        return switch (entry.state()) { case "claimable" -> 0; case "in-progress" -> 1; case "claimed" -> 2; default -> 3; };
    }
    private static String category(AchievementMenuEntry entry) { return entry.category(); }
    private static String reason(AchievementMenuEntry entry) {
        return entry.disabledReason().isBlank() ? "menu.achievements.disabled.not-claimable" : entry.disabledReason();
    }
    private static long count(List<AchievementMenuEntry> entries, String state) { return entries.stream().filter(e -> e.state().equals(state)).count(); }
    private static MenuAction empty() { return new MenuAction.Disabled("menu.disabled.no-achievements"); }
    private static MenuSpec menu(String id, String title, TreeMap<Integer, SlotSpec> slots) {
        MenuChrome.applyBorder(slots, MenuTheme.PROFILE);
        return new MenuSpec(new MenuId(id), new MenuTitle(title), new MenuSize(54), new ArrayList<>(slots.values()));
    }
    private static SlotSpec slot(int slot, String material, String key, MenuAction action, ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }
}
