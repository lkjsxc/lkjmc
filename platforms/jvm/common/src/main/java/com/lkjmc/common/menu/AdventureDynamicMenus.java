package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.TreeMap;

public final class AdventureDynamicMenus {
    private static final List<Integer> ENTRY = List.of(19, 20, 21, 22, 23, 24, 25, 28);
    private AdventureDynamicMenus() {}

    public static MenuSpec loading() {
        return LoadingDynamicMenus.loading(new MenuId("adventures"),
            "menu.adventures.title", MenuTheme.ROOT, "root");
    }

    public static MenuSpec catalog(List<AdventureMenuEntry> entries) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "DRAGON_EGG", "menu.adventures.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.adventures.info.lore"));
        var values = entries == null ? List.<AdventureMenuEntry>of() : entries.stream()
            .sorted(Comparator.comparing(AdventureMenuEntry::id)).toList();
        for (int i = 0; i < values.size() && i < ENTRY.size(); i++) {
            var entry = values.get(i);
            slots.put(ENTRY.get(i), adventure(ENTRY.get(i), entry));
        }
        if (values.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.adventures.empty", disabled(),
                ItemVisualRole.DISABLED, "menu.adventures.empty.lore"));
        }
        slots.put(31, cmd(31, "COMPASS", "menu.adventures.return", "endexpedition return",
            "menu.adventures.return.lore"));
        slots.put(49, MenuChrome.back());
        slots.put(50, MenuChrome.refresh());
        for (int border : MenuChrome.borderSlots()) {
            slots.putIfAbsent(border, MenuChrome.decoration(border, MenuTheme.ROOT));
        }
        return new MenuSpec(new MenuId("adventures"), new MenuTitle("menu.adventures.title"),
            new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec adventure(int slot, AdventureMenuEntry entry) {
        var action = entry.enabled()
            ? new MenuAction.RunPlayerCommand("buy adventure-" + entry.id())
            : disabled();
        var role = entry.enabled() ? ItemVisualRole.ACTION : ItemVisualRole.DISABLED;
        return slot(slot, entry.iconMaterial(), entry.titleKey(), action, role,
            "literal:" + entry.pricePoints() + " points");
    }

    private static SlotSpec cmd(int slot, String material, String key, String command, String... lore) {
        return slot(slot, material, key, new MenuAction.RunPlayerCommand(command), ItemVisualRole.ACTION, lore);
    }

    private static MenuAction disabled() { return new MenuAction.Disabled("menu.disabled.adventures"); }

    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }
}
