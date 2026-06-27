package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class MailDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private MailDynamicMenus() {}

    public static MenuSpec mail(List<MailMenuEntry> entries) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "WRITABLE_BOOK", "menu.mail.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.mail.info.lore"));
        var messages = entries == null ? List.<MailMenuEntry>of() : entries;
        for (int index = 0; index < messages.size() && index < ENTRY_SLOTS.size(); index++) {
            slots.put(ENTRY_SLOTS.get(index), mailSlot(ENTRY_SLOTS.get(index), messages.get(index)));
        }
        if (messages.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.mail.empty", disabled(),
                ItemVisualRole.DISABLED, "menu.mail.empty.lore"));
        }
        slots.put(49, open(49, "ARROW", "menu.back", "social", "menu.back.lore"));
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.SOCIAL.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(new MenuId("mail"), new MenuTitle("menu.mail.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec mailSlot(int slot, MailMenuEntry entry) {
        var material = entry.read() ? "BOOK" : "WRITABLE_BOOK";
        return slot(slot, material, "literal:" + entry.senderName(),
            new MenuAction.RunPlayerCommand("mail read " + entry.id()), ItemVisualRole.ACTION,
            "literal:" + snippet(entry.body()), "menu.mail.read.lore");
    }

    private static String snippet(String text) {
        return text.length() <= 48 ? text : text.substring(0, 45) + "...";
    }

    private static MenuAction disabled() {
        return new MenuAction.Disabled("menu.disabled.no-mail");
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
