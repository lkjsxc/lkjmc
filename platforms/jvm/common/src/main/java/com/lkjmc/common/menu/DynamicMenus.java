package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.TreeMap;

public final class DynamicMenus {
    private static final List<Integer> SERVER_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private DynamicMenus() {}

    public static MenuSpec serverList(List<ServerMenuEntry> entries) {
        return serverList(entries, ServerMenuPermissions.none());
    }

    public static MenuSpec serverList(List<ServerMenuEntry> entries, ServerMenuPermissions permissions) {
        var allowed = permissions == null ? ServerMenuPermissions.none() : permissions;
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "MAP", "menu.server-list.info", MenuAction.none(), ItemVisualRole.INFO,
            "menu.server-list.info.lore"));
        var sorted = entries == null ? List.<ServerMenuEntry>of() : entries.stream()
            .sorted(Comparator.comparing(ServerMenuEntry::id)).toList();
        for (int index = 0; index < sorted.size() && index < SERVER_SLOTS.size(); index++) {
            var entry = sorted.get(index);
            slots.put(SERVER_SLOTS.get(index), serverSlot(SERVER_SLOTS.get(index), entry, allowed));
        }
        if (sorted.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.server-list.empty", disabled(),
                ItemVisualRole.DISABLED, "menu.server-list.empty.lore"));
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, MenuChrome.refresh());
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.NETWORK.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(new MenuId("server-list"), new MenuTitle("menu.server-list.title"),
            new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec serverSlot(int slot, ServerMenuEntry entry, ServerMenuPermissions permissions) {
        var name = "literal:" + entry.id() + " · " + entry.desiredState();
        var lore = "literal:" + entry.kind() + " / " + entry.observedState()
            + (entry.playerCount() == null ? "" : " / " + entry.playerCount() + " online");
        var action = serverAction(entry, permissions);
        var role = action instanceof MenuAction.RunPlayerCommand ? ItemVisualRole.ACTION : ItemVisualRole.DISABLED;
        return slot(slot, material(entry), name, action, role, lore, serverLore(action));
    }

    private static MenuAction serverAction(ServerMenuEntry entry, ServerMenuPermissions permissions) {
        if (entry.desiredState().equals("stopped") || entry.desiredState().equals("suspended")) {
            return permissions.canStart() ? command("start", entry.id()) : new MenuAction.Disabled("menu.disabled.server-start-permission");
        }
        if (entry.desiredState().equals("running")) {
            if (!permissions.canStop()) { return new MenuAction.Disabled("menu.disabled.server-stop-permission"); }
            return Integer.valueOf(0).equals(entry.playerCount()) ? command("stop", entry.id())
                : new MenuAction.Disabled("menu.disabled.server-occupied");
        }
        if (entry.desiredState().equals("starting")) {
            return new MenuAction.Disabled("menu.disabled.server-starting");
        }
        return new MenuAction.Disabled("menu.disabled.server-actions");
    }

    private static MenuAction command(String action, String id) {
        return new MenuAction.RunPlayerCommand("lkjmc server " + action + " " + id);
    }

    private static String serverLore(MenuAction action) {
        return action instanceof MenuAction.Disabled disabled ? disabled.reasonKey() : "menu.server-list.action.lore";
    }

    private static String material(ServerMenuEntry entry) {
        if (entry.healthy()) {
            return "LIME_DYE";
        }
        return switch (entry.desiredState()) {
            case "running", "starting" -> "YELLOW_DYE";
            case "suspended" -> "BLUE_DYE";
            default -> "GRAY_DYE";
        };
    }

    private static MenuAction disabled() {
        return new MenuAction.Disabled("menu.disabled.server-actions");
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
