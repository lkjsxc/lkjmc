package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class RandomTeleportDynamicMenus {
    private RandomTeleportDynamicMenus() {}

    public static MenuSpec loading() {
        return LoadingDynamicMenus.loading(new MenuId("random-teleport-confirm"),
            "menu.random-teleport.title", MenuTheme.TRAVEL, "teleports");
    }

    public static MenuSpec confirm(RandomTeleportQuote quote) {
        var slots = new TreeMap<Integer, SlotSpec>();
        var lore = "literal:" + quote.costPoints() + " pts · " + quote.minRadius()
            + "-" + quote.maxRadius() + " · " + quote.maxAttempts() + " tries";
        slots.put(13, slot(13, "ENDER_PEARL", "menu.random-teleport.confirm", MenuAction.none(),
            ItemVisualRole.INFO, lore));
        slots.put(11, confirmSlot(quote));
        slots.put(15, slot(15, "RED_WOOL", "menu.confirm.no", new MenuAction.Back(), ItemVisualRole.NAVIGATION));
        slots.put(49, MenuChrome.back());
        for (int border : borderSlots()) {
            slots.putIfAbsent(border, slot(border, MenuTheme.TRAVEL.borderMaterial(), "menu.decorative",
                MenuAction.none(), ItemVisualRole.DECORATION));
        }
        return new MenuSpec(new MenuId("random-teleport-confirm"), new MenuTitle("menu.random-teleport.title"),
            new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec confirmSlot(RandomTeleportQuote quote) {
        if (quote.cooldownRemainingSeconds() > 0) {
            return disabled("menu.random-teleport.disabled.cooldown",
                "literal:" + quote.cooldownRemainingSeconds() + "s remaining");
        }
        if (!quote.canAfford()) {
            return disabled("menu.random-teleport.disabled.unaffordable",
                "literal:" + quote.costPoints() + " pts needed · " + quote.balance() + " balance");
        }
        if (!quote.enabled()) {
            return disabled("menu.random-teleport.disabled.policy", "menu.random-teleport.disabled.policy");
        }
        var lore = "literal:Cost " + quote.costPoints() + " pts; balance " + quote.balance();
        return slot(11, "LIME_WOOL", "menu.confirm.yes", new MenuAction.RunPlayerCommand("rtp confirm"),
            ItemVisualRole.SUCCESS, lore);
    }

    private static SlotSpec disabled(String reason, String lore) {
        return slot(11, "GRAY_WOOL", "menu.confirm.yes", new MenuAction.Disabled(reason),
            ItemVisualRole.DISABLED, lore);
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
