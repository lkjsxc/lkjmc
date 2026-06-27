package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

public final class ReportDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private ReportDynamicMenus() {}

    public static MenuSpec reports(List<ReportMenuEntry> reports) {
        return reports(reports, true);
    }

    public static MenuSpec reports(List<ReportMenuEntry> reports, boolean allowed) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "REDSTONE_TORCH", "menu.reports.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.reports.info.lore"));
        if (!allowed) {
            slots.put(22, slot(22, "BARRIER", "menu.reports.denied", denied(),
                ItemVisualRole.DISABLED, "menu.reports.denied.lore"));
        } else {
            var entries = reports == null ? List.<ReportMenuEntry>of() : reports;
            for (int index = 0; index < entries.size() && index < ENTRY_SLOTS.size(); index++) {
                slots.put(ENTRY_SLOTS.get(index), reportSlot(ENTRY_SLOTS.get(index), entries.get(index)));
            }
            if (entries.isEmpty()) {
                slots.put(22, slot(22, "BARRIER", "menu.reports.empty", empty(),
                    ItemVisualRole.DISABLED, "menu.reports.empty.lore"));
            }
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        addBorder(slots);
        return new MenuSpec(new MenuId("reports"), new MenuTitle("menu.reports.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    public static MenuSpec reportDetail(ReportMenuEntry report) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "REDSTONE_TORCH", "literal:report " + report.shortId(), MenuAction.none(),
            ItemVisualRole.INFO, "literal:" + report.serverId() + " / " + report.status(), "literal:" + snippet(report.reason())));
        slots.put(20, actionSlot(20, "LIME_WOOL", "menu.reports.resolve", "resolve", report.id()));
        slots.put(24, actionSlot(24, "RED_WOOL", "menu.reports.dismiss", "dismiss", report.id()));
        slots.put(49, MenuChrome.back());
        addBorder(slots);
        return new MenuSpec(new MenuId("report-detail"), new MenuTitle("menu.reports.detail.title"),
            new MenuSize(54), new ArrayList<>(slots.values()));
    }

    public static MenuSpec reportConfirm(String action, String reportId) {
        var normalized = action == null || action.isBlank() ? "resolve" : action;
        var key = normalized.equals("dismiss") ? "menu.reports.confirm.dismiss" : "menu.reports.confirm.resolve";
        return StandardMenus.confirmation(new ConfirmationSpec(new MenuId("report-confirm"), key,
            new MenuAction.RunPlayerCommand("reports " + normalized + " " + reportId)));
    }

    private static SlotSpec reportSlot(int slot, ReportMenuEntry report) {
        return slot(slot, "REDSTONE_TORCH", "literal:report " + report.shortId(),
            new MenuAction.OpenRoute(route("report-detail", report)), ItemVisualRole.NAVIGATION,
            "literal:" + report.serverId() + " / " + report.status(), "literal:" + snippet(report.reason()),
            "menu.reports.detail.lore");
    }

    private static SlotSpec actionSlot(int slot, String material, String key, String action, String reportId) {
        return slot(slot, material, key, new MenuAction.OpenRoute(confirmRoute(action, reportId)),
            ItemVisualRole.ACTION, "menu.reports.confirm.lore");
    }

    private static MenuRoute route(String id, ReportMenuEntry report) {
        return new MenuRoute(new MenuId(id), Map.of("reportId", report.id(), "serverId", report.serverId(),
            "reason", report.reason(), "status", report.status()));
    }

    private static MenuRoute confirmRoute(String action, String reportId) {
        return new MenuRoute(new MenuId("report-confirm"), Map.of("reportId", reportId, "action", action));
    }

    private static String snippet(String text) {
        return text.length() <= 48 ? text : text.substring(0, 45) + "...";
    }

    private static MenuAction denied() { return new MenuAction.Disabled("menu.disabled.reports-permission"); }
    private static MenuAction empty() { return new MenuAction.Disabled("menu.disabled.no-reports"); }

    private static SlotSpec open(int slot, String material, String key, String menu, String... lore) {
        return slot(slot, material, key, new MenuAction.OpenRoute(new MenuRoute(new MenuId(menu))),
            ItemVisualRole.NAVIGATION, lore);
    }

    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }

    private static void addBorder(TreeMap<Integer, SlotSpec> slots) {
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.SOCIAL.borderMaterial(), "menu.decorative",
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
