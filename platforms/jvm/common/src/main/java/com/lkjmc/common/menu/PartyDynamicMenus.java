package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class PartyDynamicMenus {
    private PartyDynamicMenus() {}

    public static MenuSpec loading() {
        return party(PartyStatus.loading());
    }

    public static MenuSpec party(PartyStatus status) {
        var current = status == null ? PartyStatus.loading() : status;
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "NAME_TAG", "menu.party.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.party.info.lore"));
        slots.put(20, statusSlot(current));
        slots.put(22, slot(22, "LIME_DYE", "menu.party.create", disabled("menu.disabled.party-input"),
            ItemVisualRole.DISABLED, "menu.party.create.lore"));
        slots.put(24, slot(24, "PAPER", "menu.party.invite", disabled("menu.disabled.party-picker"),
            ItemVisualRole.DISABLED, "menu.party.invite.lore"));
        slots.put(31, leaveSlot(current));
        slots.put(49, open(49, "ARROW", "menu.back", "social", "menu.back.lore"));
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(),
            ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.SOCIAL.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(new MenuId("party"), new MenuTitle("menu.party.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    public static MenuSpec partyConfirm() {
        return StandardMenus.confirmation(new ConfirmationSpec(new MenuId("party-confirm"),
            "menu.party.confirm.leave", new MenuAction.RunPlayerCommand("party leave")));
    }

    private static SlotSpec statusSlot(PartyStatus status) {
        if (!status.loaded()) {
            return slot(20, "CLOCK", "menu.party.loading", disabled("menu.disabled.party-loading"),
                ItemVisualRole.DISABLED, "menu.party.loading.lore");
        }
        if (!status.found()) {
            return slot(20, "BARRIER", "menu.party.none", MenuAction.none(), ItemVisualRole.INFO,
                "menu.party.none.lore");
        }
        return slot(20, "NAME_TAG", "literal:" + status.name(), MenuAction.none(), ItemVisualRole.INFO,
            "literal:" + status.role());
    }

    private static SlotSpec leaveSlot(PartyStatus status) {
        if (status.loaded() && status.found()) {
            return slot(31, "RED_DYE", "menu.party.leave",
                new MenuAction.OpenRoute(new MenuRoute(new MenuId("party-confirm"))),
                ItemVisualRole.ACTION, "menu.party.leave.lore");
        }
        return slot(31, "RED_DYE", "menu.party.leave", disabled("menu.disabled.no-party"),
            ItemVisualRole.DISABLED, "menu.party.leave.lore");
    }

    private static MenuAction disabled(String reason) { return new MenuAction.Disabled(reason); }
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
