package com.lkjmc.common.menu;

import com.lkjmc.common.player.GeneratedNamePolicy;
import com.lkjmc.common.player.HomeNamePolicy;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

public final class TravelDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private TravelDynamicMenus() {}

    public static MenuSpec homes(List<TravelMenuEntry> entries) {
        var slots = travelSlots(entries, "home", "RED_BED", "menu.homes.empty", "menu.homes.empty.lore");
        slots.put(45, slot(45, "LIME_BED", "menu.homes.set",
            new MenuAction.OpenRoute(new MenuRoute(new MenuId("home-create-confirm"), Map.of("home", nextHome(entries)))),
            ItemVisualRole.ACTION, "menu.homes.set.lore"));
        return menu("homes", "menu.homes.title", slots);
    }

    public static MenuSpec homeDetail(String home, String serverId) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "RED_BED", "literal:" + home, MenuAction.none(), ItemVisualRole.INFO,
            "literal:" + serverId));
        slots.put(20, slot(20, "ENDER_PEARL", "menu.homes.teleport",
            new MenuAction.RunPlayerCommand("home " + home), ItemVisualRole.ACTION, "menu.homes.teleport.lore"));
        slots.put(22, slot(22, "LIME_BED", "menu.homes.update",
            new MenuAction.OpenRoute(new MenuRoute(new MenuId("home-update-confirm"), Map.of("home", home))),
            ItemVisualRole.ACTION, "menu.homes.update.lore"));
        slots.put(24, slot(24, "TNT", "menu.homes.delete",
            new MenuAction.OpenRoute(new MenuRoute(new MenuId("home-delete-confirm"), Map.of("home", home))),
            ItemVisualRole.ACTION, "menu.homes.delete.lore"));
        slots.put(49, MenuChrome.back());
        slots.put(45, MenuChrome.mainMenu(45));
        return menu("home-detail", "menu.homes.detail.title", slots);
    }

    public static MenuSpec homeUpdateConfirm(String home) {
        return confirm("home-update-confirm", "menu.homes.update.confirm", "LIME_BED",
            new MenuAction.DaemonCommand("player.home.set", MenuActionPayload.of("home", home)));
    }

    public static MenuSpec homeDeleteConfirm(String home) {
        return confirm("home-delete-confirm", "menu.homes.delete.confirm", "TNT",
            new MenuAction.DaemonCommand("player.home.delete", MenuActionPayload.of("home", home)));
    }

    public static MenuSpec homeCreateName(List<TravelMenuEntry> entries) {
        return homeCreateConfirm(new MenuId("home-create-name"), nextHome(entries));
    }

    public static MenuSpec homeCreateConfirm(String home) {
        return homeCreateConfirm(new MenuId("home-create-confirm"), home);
    }

    private static MenuSpec homeCreateConfirm(MenuId id, String home) {
        return confirm(id.value(), "menu.homes.set", "LIME_BED",
            new MenuAction.DaemonCommand("player.home.set", MenuActionPayload.of("home", home)));
    }

    public static MenuSpec warps(List<TravelMenuEntry> entries) {
        return menu("warps", "menu.warps.title", travelSlots(entries, "warp", "OAK_SIGN",
            "menu.warps.empty", "menu.warps.empty.lore"));
    }

    private static MenuSpec confirm(String id, String title, String material, MenuAction action) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(13, slot(13, material, title, MenuAction.none(), ItemVisualRole.INFO));
        slots.put(11, slot(11, "LIME_WOOL", "menu.confirm.yes", action, ItemVisualRole.SUCCESS));
        slots.put(15, slot(15, "RED_WOOL", "menu.confirm.no", new MenuAction.Back(), ItemVisualRole.NAVIGATION));
        return menu(id, title, slots);
    }

    private static MenuSpec menu(String id, String title, TreeMap<Integer, SlotSpec> slots) {
        MenuChrome.applyBorder(slots, MenuTheme.TRAVEL);
        return new MenuSpec(new MenuId(id), new MenuTitle(title), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static TreeMap<Integer, SlotSpec> travelSlots(List<TravelMenuEntry> entries, String command,
                                                          String material, String emptyKey, String emptyLore) {
        var slots = new TreeMap<Integer, SlotSpec>();
        var sorted = entries == null ? List.<TravelMenuEntry>of() : entries.stream()
            .sorted(Comparator.comparing(TravelMenuEntry::name)).toList();
        for (int index = 0; index < sorted.size() && index < ENTRY_SLOTS.size(); index++) {
            slots.put(ENTRY_SLOTS.get(index), entrySlot(ENTRY_SLOTS.get(index), sorted.get(index), command, material));
        }
        if (sorted.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", emptyKey, disabled(), ItemVisualRole.DISABLED, emptyLore));
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, MenuChrome.refresh());
        return slots;
    }

    private static String nextHome(List<TravelMenuEntry> entries) {
        return GeneratedNamePolicy.nextNumbered("home", existing(entries));
    }

    private static Set<String> existing(List<TravelMenuEntry> entries) {
        if (entries == null) { return Set.of(); }
        return entries.stream().map(TravelMenuEntry::name).collect(java.util.stream.Collectors.toUnmodifiableSet());
    }

    private static SlotSpec entrySlot(int slot, TravelMenuEntry entry, String command, String material) {
        if (command.equals("home") && !HomeNamePolicy.isValid(entry.name())) {
            return slot(slot, "BARRIER", "literal:" + entry.name(),
                new MenuAction.Disabled("menu.disabled.invalid-home-name"), ItemVisualRole.DISABLED,
                "literal:" + entry.serverId(), "menu.disabled.invalid-home-name");
        }
        if (command.equals("home")) {
            return slot(slot, material, "literal:" + entry.name(),
                new MenuAction.OpenRoute(new MenuRoute(new MenuId("home-detail"),
                    Map.of("home", entry.name(), "serverId", entry.serverId()))),
                ItemVisualRole.ACTION, "literal:" + entry.serverId(), "menu.homes.detail.lore");
        }
        return slot(slot, material, "literal:" + entry.name(),
            new MenuAction.RunPlayerCommand(command + " " + entry.name()), ItemVisualRole.ACTION,
            "literal:" + entry.serverId(), "menu.travel.teleport.lore");
    }

    private static MenuAction disabled() { return new MenuAction.Disabled("menu.disabled.no-travel-entries"); }
    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }
}
