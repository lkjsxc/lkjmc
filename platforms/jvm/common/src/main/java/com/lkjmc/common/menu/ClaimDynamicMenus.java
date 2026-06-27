package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

public final class ClaimDynamicMenus {
    private static final List<Integer> ENTRY_SLOTS = List.of(19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34);

    private ClaimDynamicMenus() {}

    public static MenuSpec claims(List<ClaimMenuEntry> entries) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "FILLED_MAP", "menu.claims.info", MenuAction.none(), ItemVisualRole.INFO,
            "menu.claims.info.lore"));
        slots.put(40, slot(40, "GOLDEN_SHOVEL", "menu.claims.create",
            new MenuAction.TextInput("menu.input.claim-name.prompt", "claim create"), ItemVisualRole.ACTION,
            "menu.claims.create.lore"));
        var sorted = entries == null ? List.<ClaimMenuEntry>of() : entries.stream()
            .sorted(Comparator.comparing(ClaimMenuEntry::name)).toList();
        for (int index = 0; index < sorted.size() && index < ENTRY_SLOTS.size(); index++) {
            slots.put(ENTRY_SLOTS.get(index), claimSlot(ENTRY_SLOTS.get(index), sorted.get(index)));
        }
        if (sorted.isEmpty()) {
            slots.put(22, slot(22, "BARRIER", "menu.claims.empty", disabled(),
                ItemVisualRole.DISABLED, "menu.claims.empty.lore"));
        }
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        addBorder(slots);
        return new MenuSpec(new MenuId("claims"), new MenuTitle("menu.claims.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    public static MenuSpec claimDetail(String name, long chunkCount) {
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "FILLED_MAP", "literal:" + name, MenuAction.none(), ItemVisualRole.INFO,
            "literal:" + chunkCount + " chunks"));
        slots.put(20, slot(20, "RED_WOOL", "menu.claims.delete", new MenuAction.OpenRoute(confirmRoute(name)),
            ItemVisualRole.ACTION, "menu.claims.delete.lore"));
        slots.put(24, slot(24, "PLAYER_HEAD", "menu.claims.trust",
            new MenuAction.OpenRoute(new MenuRoute(new MenuId("claim-trust-picker"), Map.of("name", name))),
            ItemVisualRole.NAVIGATION, "menu.claims.trust.lore"));
        slots.put(49, MenuChrome.back());
        addBorder(slots);
        return new MenuSpec(new MenuId("claim-detail"), new MenuTitle("menu.claims.detail.title"),
            new MenuSize(54), new ArrayList<>(slots.values()));
    }

    public static MenuSpec claimConfirm(String name) {
        return StandardMenus.confirmation(new ConfirmationSpec(new MenuId("claim-confirm"),
            "menu.claims.confirm.delete", new MenuAction.RunPlayerCommand("claim delete " + name)));
    }

    private static SlotSpec claimSlot(int slot, ClaimMenuEntry entry) {
        return slot(slot, "FILLED_MAP", "literal:" + entry.name(),
            new MenuAction.OpenRoute(detailRoute(entry)), ItemVisualRole.NAVIGATION,
            "literal:" + entry.chunkCount() + " chunks", "menu.claims.detail.lore");
    }

    private static MenuRoute detailRoute(ClaimMenuEntry entry) {
        return new MenuRoute(new MenuId("claim-detail"), Map.of(
            "name", entry.name(), "chunkCount", Long.toString(entry.chunkCount())));
    }

    private static MenuRoute confirmRoute(String name) {
        return new MenuRoute(new MenuId("claim-confirm"), Map.of("name", name));
    }

    private static MenuAction disabled() { return new MenuAction.Disabled("menu.disabled.no-claims"); }
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
            slots.putIfAbsent(border, slot(border, MenuTheme.CLAIMS.borderMaterial(), "menu.decorative",
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
