package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class StandardMenus {
    private StandardMenus() {}

    public static MenuRegistry registry() {
        return new MenuRegistry(List.of(root(), network(), travel(), homes(), warps(), teleports(),
            claims(), economy(), shopList(), shopDetail(), kits(), daily(), votes(), social(), party(),
            mail(), reports(), reportDetail(), reportConfirm(), profile(), achievements(), settings(), language(),
            serverList(), serverDetail()));
    }

    public static MenuSpec root() {
        return menu("root", "menu.root.title", MenuTheme.ROOT, List.of(
            slot(4, "PAPER", "menu.root.info", inert(), ItemVisualRole.INFO, "menu.root.info.lore"),
            open(19, "COMPASS", "menu.network.title", "network", "menu.network.lore"),
            open(20, "ENDER_PEARL", "menu.travel.title", "travel", "menu.travel.lore"),
            open(21, "GOLDEN_SHOVEL", "menu.claims.title", "claims", "menu.claims.lore"),
            open(22, "EMERALD", "menu.economy.title", "economy", "menu.economy.lore"),
            open(23, "WRITABLE_BOOK", "menu.social.title", "social", "menu.social.lore"),
            open(24, "PLAYER_HEAD", "menu.profile.title", "profile", "menu.profile.lore"),
            open(25, "COMPARATOR", "menu.settings.title", "settings", "menu.settings.lore"),
            disabled(31, "RED_DYE", "menu.staff.title", "menu.disabled.staff", "menu.staff.lore"),
            disabled(40, "DRAGON_EGG", "menu.adventures.title", "menu.disabled.adventures", "menu.adventures.lore"),
            slot(50, "BARRIER", "menu.close", new MenuAction.Close(), ItemVisualRole.NAVIGATION)));
    }

    public static MenuSpec network() {
        return menu("network", "menu.network.title", MenuTheme.NETWORK, List.of(
            slot(4, "MAP", "menu.network.info", inert(), ItemVisualRole.INFO, "menu.network.info.lore"),
            open(20, "LECTERN", "menu.server-list.title", "server-list", "menu.server-list.lore"),
            cmd(24, "COMPASS", "menu.network.hub", "lkjmc", "menu.network.hub.lore"), back()));
    }

    public static MenuSpec serverList() { return loading("server-list", "menu.server-list.title", MenuTheme.NETWORK, "network"); }

    public static MenuSpec serverDetail() {
        return disabledMenu("server-detail", "menu.server-detail.title", MenuTheme.NETWORK,
            "menu.disabled.select-server", "server-list");
    }

    public static MenuSpec travel() {
        return menu("travel", "menu.travel.title", MenuTheme.TRAVEL, List.of(
            open(20, "RED_BED", "menu.homes.title", "homes", "menu.homes.lore"),
            open(22, "OAK_SIGN", "menu.warps.title", "warps", "menu.warps.lore"),
            open(24, "ENDER_PEARL", "menu.teleports.title", "teleports", "menu.teleports.lore"), back()));
    }

    public static MenuSpec homes() { return loading("homes", "menu.homes.title", MenuTheme.TRAVEL, "travel"); }
    public static MenuSpec warps() { return loading("warps", "menu.warps.title", MenuTheme.TRAVEL, "travel"); }
    public static MenuSpec teleports() { return TeleportDynamicMenus.teleports(); }

    public static MenuSpec claims() { return loading("claims", "menu.claims.title", MenuTheme.CLAIMS, "root"); }

    public static MenuSpec economy() {
        return menu("economy", "menu.economy.title", MenuTheme.ECONOMY, List.of(
            cmd(19, "EMERALD", "menu.points.title", "points", "menu.points.lore"),
            open(20, "CHEST", "menu.shop.title", "shop", "menu.shop.lore"),
            open(21, "IRON_SWORD", "menu.kits.title", "kits", "menu.kits.lore"),
            open(22, "SUNFLOWER", "menu.daily.title", "daily", "menu.daily.lore"),
            open(23, "PAPER", "menu.votes.title", "votes", "menu.votes.lore"), back()));
    }

    public static MenuSpec shopList() { return loading("shop", "menu.shop.title", MenuTheme.ECONOMY, "economy"); }
    public static MenuSpec shopDetail() { return disabledMenu("shop-detail", "menu.shop-detail.title", MenuTheme.ECONOMY, "menu.disabled.select-shop-item", "shop"); }
    public static MenuSpec kits() { return loading("kits", "menu.kits.title", MenuTheme.ECONOMY, "economy"); }
    public static MenuSpec daily() { return DailyDynamicMenus.loading(); }
    public static MenuSpec votes() { return loading("votes", "menu.votes.title", MenuTheme.ECONOMY, "economy"); }

    public static MenuSpec social() { return menu("social", "menu.social.title", MenuTheme.SOCIAL, List.of(open(20, "WRITABLE_BOOK", "menu.mail.title", "mail", "menu.mail.lore"), open(22, "NAME_TAG", "menu.party.title", "party", "menu.party.lore"), open(24, "REDSTONE_TORCH", "menu.reports.title", "reports", "menu.reports.lore"), back())); }
    public static MenuSpec party() { return PartyDynamicMenus.loading(); }
    public static MenuSpec mail() { return loading("mail", "menu.mail.title", MenuTheme.SOCIAL, "social"); }
    public static MenuSpec reports() { return loading("reports", "menu.reports.title", MenuTheme.SOCIAL, "social"); }
    public static MenuSpec reportDetail() { return loading("report-detail", "menu.reports.detail.title", MenuTheme.SOCIAL, "reports"); }
    public static MenuSpec reportConfirm() { return loading("report-confirm", "menu.reports.confirm.title", MenuTheme.SOCIAL, "report-detail"); }
    public static MenuSpec profile() { return ProfileDynamicMenus.loading(); }
    public static MenuSpec achievements() { return AchievementDynamicMenus.loading(); }
    public static MenuSpec settings() { return menu("settings", "menu.settings.title", MenuTheme.SETTINGS, List.of(open(20, "BOOK", "menu.language.title", "language", "menu.language.lore"), daemon(22, "GLOWSTONE_DUST", "menu.hud.toggle", "player.settings.toggle", "settingKey=hud", "menu.hud.toggle.lore"), daemon(24, "COMPASS", "menu.hotbar-token.toggle", "player.settings.toggle", "settingKey=menu-token", "menu.hotbar-token.toggle.lore"), back())); }
    public static MenuSpec language() { return menu("language", "menu.language.title", MenuTheme.SETTINGS, List.of(daemon(20, "PAPER", "language.english", "player.settings.set", "language=en", "language.english.lore"), daemon(24, "PAPER", "language.japanese", "player.settings.set", "language=ja", "language.japanese.lore"), backTo("settings"))); }

    public static MenuSpec confirmation(ConfirmationSpec spec) {
        return new MenuSpec(spec.id(), new MenuTitle(spec.messageKey()), new MenuSize(27), List.of(
            slot(11, "LIME_WOOL", "menu.confirm.yes", spec.confirmAction(), ItemVisualRole.SUCCESS),
            slot(15, "RED_WOOL", "menu.confirm.no", new MenuAction.Back(), ItemVisualRole.NAVIGATION)));
    }

    public static NavigationPolicy navigation() { return NavigationPolicy.standard54(); }
    private static MenuSpec loading(String id, String title, MenuTheme theme, String back) { return LoadingDynamicMenus.loading(new MenuId(id), title, theme, back); }
    private static MenuSpec commandMenu(String id, String title, MenuTheme theme, String material, String key, String command, String lore, String back) { return menu(id, title, theme, List.of(cmd(22, material, key, command, lore), backTo(back))); }
    private static MenuSpec disabledMenu(String id, String title, MenuTheme theme, String reason, String back) { return menu(id, title, theme, List.of(disabled(22, "BARRIER", reason, reason, "menu.disabled.lore"), backTo(back))); }
    private static MenuSpec menu(String id, String title, MenuTheme theme, List<SlotSpec> functional) { var slots = new TreeMap<Integer, SlotSpec>(); for (var slot : functional) { slots.put(slot.slot(), slot); } for (int border : borderSlots()) { slots.putIfAbsent(border, pane(border, theme)); } return new MenuSpec(new MenuId(id), new MenuTitle(title), new MenuSize(54), new ArrayList<>(slots.values())); }
    private static SlotSpec open(int slot, String material, String key, String menu, String... lore) { return slot(slot, material, key, new MenuAction.OpenRoute(new MenuRoute(new MenuId(menu))), ItemVisualRole.NAVIGATION, lore); }
    private static SlotSpec cmd(int slot, String material, String key, String command, String... lore) { return slot(slot, material, key, new MenuAction.RunPlayerCommand(command), ItemVisualRole.ACTION, lore); }
    private static SlotSpec daemon(int slot, String material, String key, String command, String payload, String... lore) { return slot(slot, material, key, new MenuAction.DaemonCommand(command, new MenuActionPayload(payload)), ItemVisualRole.ACTION, lore); }
    private static SlotSpec disabled(int slot, String material, String key, String reason, String... lore) { return slot(slot, material, key, new MenuAction.Disabled(reason), ItemVisualRole.DISABLED, lore); }
    private static SlotSpec back() { return slot(49, "ARROW", "menu.back", new MenuAction.Back(), ItemVisualRole.NAVIGATION, "menu.back.lore"); }
    private static SlotSpec backTo(String id) { return open(49, "ARROW", "menu.back", id, "menu.back.lore"); }
    private static SlotSpec refresh() { return slot(50, "CLOCK", "menu.refresh", new MenuAction.RefreshRoute(), ItemVisualRole.NAVIGATION, "menu.refresh.lore"); }
    private static SlotSpec pane(int slot, MenuTheme theme) { return slot(slot, theme.borderMaterial(), "menu.decorative", inert(), ItemVisualRole.DECORATION); }
    private static MenuAction inert() { return MenuAction.none(); }
    private static SlotSpec slot(int slot, String material, String key, MenuAction action, ItemVisualRole role, String... lore) { return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action); }
    private static List<Integer> borderSlots() { var slots = new ArrayList<Integer>(); for (int i = 0; i <= 8; i++) { slots.add(i); } for (int i = 45; i <= 53; i++) { slots.add(i); } slots.addAll(List.of(9, 18, 27, 36, 17, 26, 35, 44)); return slots; }
}
