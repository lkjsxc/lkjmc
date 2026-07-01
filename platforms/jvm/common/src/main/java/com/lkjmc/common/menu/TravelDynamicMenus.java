package com.lkjmc.common.menu;

import com.lkjmc.common.player.HomeNamePolicy;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

public final class TravelDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);
    private static final List<String> HOME_NAMES = List.of("home", "base", "mine", "farm", "village", "nether", "end");

    private TravelDynamicMenus() {}

    public static MenuSpec homes(List<TravelMenuEntry> entries) {
        var slots = travelSlots(entries, "home", "RED_BED", "menu.homes.empty", "menu.homes.empty.lore");
        slots.put(10, slot(10, "LIME_BED", "menu.homes.set", new MenuAction.OpenRoute(new MenuRoute(new MenuId("home-create-name"))),
            ItemVisualRole.ACTION, "menu.homes.set.lore"));
        return menu("homes", "menu.homes.title", slots);
    }

    public static MenuSpec homeCreateName(List<TravelMenuEntry> entries) {
        var slots = new TreeMap<Integer, SlotSpec>();
        var existing = existing(entries);
        for (int i = 0; i < HOME_NAMES.size(); i++) {
            var name = HOME_NAMES.get(i);
            var action = existing.contains(name) ? new MenuAction.Disabled("menu.disabled.invalid-home-name")
                : new MenuAction.OpenRoute(new MenuRoute(new MenuId("home-create-confirm"), Map.of("home", name)));
            slots.put(ENTRY_SLOTS.get(i), slot(ENTRY_SLOTS.get(i), existing.contains(name) ? "GRAY_BED" : "LIME_BED",
                "literal:" + name, action, existing.contains(name) ? ItemVisualRole.DISABLED : ItemVisualRole.ACTION,
                existing.contains(name) ? "menu.disabled.invalid-home-name" : "menu.homes.set.lore"));
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, MenuChrome.refresh());
        addBorder(slots);
        return new MenuSpec(new MenuId("home-create-name"), new MenuTitle("menu.homes.set"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    public static MenuSpec homeCreateConfirm(String home) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(13, slot(13, "LIME_BED", "literal:Set home " + home, MenuAction.none(), ItemVisualRole.INFO));
        slots.put(11, slot(11, "LIME_WOOL", "menu.confirm.yes",
            new MenuAction.DaemonCommand("player.home.set", MenuActionPayload.of("home", home)), ItemVisualRole.SUCCESS));
        slots.put(15, slot(15, "RED_WOOL", "menu.confirm.no", new MenuAction.Back(), ItemVisualRole.NAVIGATION));
        addBorder(slots);
        return new MenuSpec(new MenuId("home-create-confirm"), new MenuTitle("menu.homes.set"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    public static MenuSpec warps(List<TravelMenuEntry> entries) {
        return menu("warps", "menu.warps.title", entries, "warp", "OAK_SIGN", "menu.warps.empty",
            "menu.warps.empty.lore", "travel");
    }

    private static MenuSpec menu(String id, String title, List<TravelMenuEntry> entries,
                                 String command, String material, String emptyKey,
                                 String emptyLore, String back) {
        return menu(id, title, travelSlots(entries, command, material, emptyKey, emptyLore));
    }

    private static MenuSpec menu(String id, String title, TreeMap<Integer, SlotSpec> slots) {
        addBorder(slots);
        return new MenuSpec(new MenuId(id), new MenuTitle(title), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static TreeMap<Integer, SlotSpec> travelSlots(List<TravelMenuEntry> entries, String command,
                                                          String material, String emptyKey, String emptyLore) {
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
        slots.put(50, MenuChrome.refresh());
        return slots;
    }

    private static Set<String> existing(List<TravelMenuEntry> entries) {
        if (entries == null) {
            return Set.of();
        }
        return entries.stream().map(TravelMenuEntry::name).collect(java.util.stream.Collectors.toUnmodifiableSet());
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

    private static void addBorder(TreeMap<Integer, SlotSpec> slots) {
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.TRAVEL.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
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
