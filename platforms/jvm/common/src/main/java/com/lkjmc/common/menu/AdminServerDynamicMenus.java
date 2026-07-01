package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

public final class AdminServerDynamicMenus {
    private static final List<Integer> ENTRY = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private AdminServerDynamicMenus() {}

    public static MenuSpec servers(List<ServerMenuEntry> entries, AdminMenuPermissions p) {
        var slots = base();
        slots.put(4, slot(4, "LECTERN", "menu.server-list.info", MenuAction.none(), ItemVisualRole.INFO,
            "menu.server-list.info.lore"));
        slots.put(40, createSlot(p));
        var sorted = entries == null ? List.<ServerMenuEntry>of()
            : entries.stream().sorted(Comparator.comparing(ServerMenuEntry::id)).toList();
        for (var i = 0; i < sorted.size() && i < ENTRY.size(); i++) {
            slots.put(ENTRY.get(i), row(ENTRY.get(i), sorted.get(i)));
        }
        if (sorted.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.server-list.empty", disabled("menu.disabled.select-server"),
                ItemVisualRole.DISABLED, "menu.server-list.empty.lore"));
        }
        return menu("admin-servers", "menu.admin.servers.title", slots);
    }

    public static MenuSpec detail(ServerMenuEntry entry, AdminMenuPermissions p) {
        var slots = base();
        slots.put(4, slot(4, "LECTERN", "literal:" + entry.id(), MenuAction.none(), ItemVisualRole.INFO,
            "literal:" + summary(entry)));
        slots.put(19, daemon(19, "LIME_WOOL", "menu.admin.server.start", "instance.start", entry.id(), p.startServer()));
        slots.put(20, confirm(20, "ORANGE_WOOL", "menu.admin.server.stop", "admin-server-stop-confirm", entry.id(), p.stopServer()));
        slots.put(21, confirm(21, "ANVIL", "menu.admin.server.restart", "admin-server-restart-confirm", entry.id(), p.restartServer()));
        slots.put(22, daemon(22, "PAPER", "menu.admin.audit.tail", "instance.logs", entry.id(), p.listServers()));
        slots.put(24, confirm(24, "BARRIER", "menu.admin.server.delete", "admin-server-delete-confirm", entry.id(), p.deleteServer()));
        return menu("admin-server-detail", "menu.server-detail.title", slots);
    }

    public static MenuSpec confirm(String id, String action, String command) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(13, slot(13, "OAK_SIGN", "literal:" + action + " " + id, MenuAction.none(), ItemVisualRole.INFO));
        slots.put(11, slot(11, "LIME_WOOL", "menu.confirm.yes",
            new MenuAction.DaemonCommand(command, new MenuActionPayload(Map.of("id", id, "force", "false"))),
            ItemVisualRole.SUCCESS));
        slots.put(15, slot(15, "RED_WOOL", "menu.confirm.no", new MenuAction.Back(), ItemVisualRole.NAVIGATION));
        return menu("admin-server-" + action + "-confirm", "menu.confirm.yes", slots);
    }

    private static SlotSpec row(int slot, ServerMenuEntry entry) {
        return slot(slot, entry.healthy() ? "GREEN_WOOL" : "YELLOW_WOOL", "literal:" + entry.id() + " · " + entry.desiredState(),
            new MenuAction.OpenRoute(route(entry)), ItemVisualRole.NAVIGATION, "literal:" + summary(entry));
    }

    private static MenuRoute route(ServerMenuEntry entry) {
        return new MenuRoute(new MenuId("admin-server-detail"), Map.of("id", entry.id()));
    }

    private static SlotSpec createSlot(AdminMenuPermissions p) {
        var action = p.createServer()
            ? new MenuAction.TextInput("menu.admin.input.server-create", "lkjmc server create ")
            : disabled("menu.disabled.admin-permission");
        return slot(40, "NAME_TAG", "menu.admin.server.create", action,
            p.createServer() ? ItemVisualRole.ACTION : ItemVisualRole.DISABLED);
    }

    private static SlotSpec daemon(int slot, String material, String key, String command, String id, boolean enabled) {
        var action = enabled ? new MenuAction.DaemonCommand(command, MenuActionPayload.of("id", id))
            : disabled("menu.disabled.admin-permission");
        return slot(slot, material, key, action, enabled ? ItemVisualRole.ACTION : ItemVisualRole.DISABLED);
    }

    private static SlotSpec confirm(int slot, String material, String key, String route, String id, boolean enabled) {
        var action = enabled ? new MenuAction.OpenRoute(new MenuRoute(new MenuId(route), Map.of("id", id)))
            : disabled("menu.disabled.admin-permission");
        return slot(slot, material, key, action, enabled ? ItemVisualRole.ACTION : ItemVisualRole.DISABLED);
    }

    private static String summary(ServerMenuEntry entry) {
        return entry.kind() + " desired=" + entry.desiredState() + " observed=" + entry.observedState()
            + " players=" + (entry.playerCount() == null ? "?" : entry.playerCount());
    }

    private static TreeMap<Integer, SlotSpec> base() {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(49, MenuChrome.back());
        slots.put(50, MenuChrome.refresh());
        return slots;
    }

    private static MenuSpec menu(String id, String title, TreeMap<Integer, SlotSpec> slots) {
        for (int border : MenuChrome.borderSlots()) {
            slots.putIfAbsent(border, MenuChrome.decoration(border, MenuTheme.SETTINGS));
        }
        return new MenuSpec(new MenuId(id), new MenuTitle(title), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static MenuAction disabled(String reason) { return new MenuAction.Disabled(reason); }
    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }
}
