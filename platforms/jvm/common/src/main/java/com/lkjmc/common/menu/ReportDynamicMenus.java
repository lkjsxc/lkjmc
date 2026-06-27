package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
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
        slots.put(49, open(49, "ARROW", "menu.back", "social", "menu.back.lore"));
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.SOCIAL.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(new MenuId("reports"), new MenuTitle("menu.reports.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec reportSlot(int slot, ReportMenuEntry report) {
        return slot(slot, "REDSTONE_TORCH", "literal:report " + report.shortId(),
            new MenuAction.Disabled("menu.disabled.report-confirmation"), ItemVisualRole.DISABLED,
            "literal:" + report.serverId() + " / " + report.status(), "literal:" + snippet(report.reason()),
            "menu.disabled.report-confirmation");
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

    private static List<Integer> borderSlots() {
        var slots = new ArrayList<Integer>();
        for (int i = 0; i <= 8; i++) { slots.add(i); }
        for (int i = 45; i <= 53; i++) { slots.add(i); }
        slots.addAll(List.of(9, 18, 27, 36, 17, 26, 35, 44));
        return slots;
    }
}
