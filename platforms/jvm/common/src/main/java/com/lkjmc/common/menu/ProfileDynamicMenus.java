package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class ProfileDynamicMenus {
    private ProfileDynamicMenus() {}

    public static MenuSpec loading() {
        return profile(ProfileSummary.loading());
    }

    public static MenuSpec profile(ProfileSummary summary) {
        var current = summary == null ? ProfileSummary.loading() : summary;
        var slots = new TreeMap<Integer, SlotSpec>();
        slots.put(4, slot(4, "PLAYER_HEAD", "menu.profile.info", MenuAction.none(),
            ItemVisualRole.INFO, "menu.profile.info.lore"));
        slots.put(20, pointsSlot(current));
        slots.put(22, achievementsSlot(current));
        slots.put(24, open(24, "CLOCK", "menu.profile.hud", "settings", "menu.profile.hud.lore"));
        slots.put(49, MenuChrome.back());
        slots.put(50, slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(), ItemVisualRole.NAVIGATION, "menu.refresh.lore"));
        MenuChrome.applyBorder(slots, MenuTheme.PROFILE);
        return new MenuSpec(new MenuId("profile"), new MenuTitle("menu.profile.title"), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec pointsSlot(ProfileSummary summary) {
        if (!summary.loaded()) {
            return slot(20, "CLOCK", "menu.profile.loading", disabled(), ItemVisualRole.DISABLED,
                "menu.profile.loading.lore");
        }
        return slot(20, "EMERALD", "menu.profile.points", MenuAction.none(), ItemVisualRole.INFO,
            "literal:" + summary.pointsBalance() + " points");
    }

    private static SlotSpec achievementsSlot(ProfileSummary summary) {
        if (!summary.loaded()) {
            return slot(22, "CLOCK", "menu.profile.loading", disabled(), ItemVisualRole.DISABLED,
                "menu.profile.loading.lore");
        }
        return slot(22, "DIAMOND", "menu.profile.achievements",
            new MenuAction.OpenRoute(new MenuRoute(new MenuId("achievements"))), ItemVisualRole.NAVIGATION,
            "literal:" + summary.achievementCount() + " claimed", "menu.profile.achievements.lore");
    }

    private static MenuAction disabled() { return new MenuAction.Disabled("menu.disabled.profile-loading"); }
    private static SlotSpec open(int slot, String material, String key, String menu, String... lore) {
        return slot(slot, material, key, new MenuAction.OpenRoute(new MenuRoute(new MenuId(menu))),
            ItemVisualRole.NAVIGATION, lore);
    }
    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }
}
