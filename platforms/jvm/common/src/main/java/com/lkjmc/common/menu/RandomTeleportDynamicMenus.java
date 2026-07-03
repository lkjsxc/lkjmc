package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class RandomTeleportDynamicMenus {
    private RandomTeleportDynamicMenus() {}

    public static MenuSpec loading() { return loading("overworld"); }

    public static MenuSpec loading(String profileId) {
        return LoadingDynamicMenus.loading(id(profileId), "menu.random-teleport.title", MenuTheme.TRAVEL, "teleports");
    }

    public static MenuSpec confirm(RandomTeleportQuote quote) {
        var slots = new TreeMap<Integer, SlotSpec>();
        var lore = "literal:" + quote.profileId() + " · " + quote.costPoints() + " pts · "
            + quote.minRadius() + "-" + quote.maxRadius() + " · " + quote.maxAttempts() + " tries";
        slots.put(13, slot(13, "ENDER_PEARL", "menu.random-teleport.confirm", MenuAction.none(),
            ItemVisualRole.INFO, lore));
        slots.put(11, confirmSlot(quote));
        slots.put(15, slot(15, "RED_WOOL", "menu.confirm.no", new MenuAction.Back(), ItemVisualRole.NAVIGATION));
        slots.put(49, MenuChrome.back());
        MenuChrome.applyBorder(slots, MenuTheme.TRAVEL);
        return new MenuSpec(id(quote.profileId()), new MenuTitle("menu.random-teleport.title"),
            new MenuSize(54), new ArrayList<>(slots.values()));
    }

    public static MenuId id(String profileId) {
        return switch (profileId == null ? "overworld" : profileId) {
            case "nether" -> new MenuId("random-teleport-nether-confirm");
            case "end" -> new MenuId("random-teleport-end-confirm");
            default -> new MenuId("random-teleport-overworld");
        };
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
        var command = quote.confirmationRequired() ? "rtp " + quote.profileId() + " confirm" : "rtp " + quote.profileId();
        var lore = "literal:Cost " + quote.costPoints() + " pts; balance " + quote.balance();
        return slot(11, "LIME_WOOL", quote.confirmationRequired() ? "menu.confirm.yes" : "menu.random-teleport.start",
            new MenuAction.RunPlayerCommand(command), ItemVisualRole.SUCCESS, lore);
    }

    private static SlotSpec disabled(String reason, String lore) {
        return slot(11, "GRAY_WOOL", "menu.confirm.yes", new MenuAction.Disabled(reason),
            ItemVisualRole.DISABLED, lore);
    }

    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }
}
