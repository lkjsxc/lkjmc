package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;

public final class AdminDynamicMenus {
    private AdminDynamicMenus() {}

    public static MenuSpec loading(String id) {
        return LoadingDynamicMenus.loading(new MenuId(id), title(id), MenuTheme.SETTINGS, "admin");
    }

    public static MenuSpec dashboard(AdminMenuPermissions p) {
        return menu("admin", "menu.admin.title", List.of(
            action(10, "HEART_OF_THE_SEA", "menu.admin.status", "lkjmc status", p.status(), "menu.admin.status.lore"),
            action(12, "COMPASS", "menu.admin.doctor", "lkjmc doctor", p.status(), "menu.admin.doctor.lore"),
            open(14, "LECTERN", "menu.admin.servers", "admin-servers", "menu.admin.servers.lore"),
            open(16, "REDSTONE", "menu.admin.config", "admin-config", "menu.admin.config.lore"),
            open(28, "NETHER_STAR", "menu.admin.security", "admin-security", "menu.admin.security.lore"),
            open(30, "EMERALD", "menu.admin.economy", "admin-economy", "menu.admin.economy.lore"),
            open(32, "IRON_AXE", "menu.admin.moderation", "admin-moderation", "menu.admin.moderation.lore"),
            open(34, "PAPER", "menu.admin.audit", "admin-audit", "menu.admin.audit.lore"),
            open(40, "OAK_SIGN", "menu.admin.web", "admin-web", "menu.admin.web.lore"), back()));
    }

    public static MenuSpec servers(AdminMenuPermissions p) {
        return menu("admin-servers", "menu.admin.servers.title", List.of(
            action(19, "LECTERN", "menu.server-list.title", "lkjmc server list", p.listServers(), "menu.server-list.lore"),
            input(20, "LIME_WOOL", "menu.admin.server.start", "menu.admin.input.server-start", "lkjmc server start ", p.startServer()),
            input(21, "ORANGE_WOOL", "menu.admin.server.stop", "menu.admin.input.server-stop", "lkjmc server stop ", p.stopServer()),
            input(22, "ANVIL", "menu.admin.server.restart", "menu.admin.input.server-restart", "lkjmc server restart ", p.restartServer()),
            input(23, "NAME_TAG", "menu.admin.server.create", "menu.admin.input.server-create", "lkjmc server create ", p.createServer()),
            input(24, "BARRIER", "menu.admin.server.delete", "menu.admin.input.server-delete", "lkjmc server delete ", p.deleteServer()), back()));
    }

    public static MenuSpec config(AdminMenuPermissions p) {
        return menu("admin-config", "menu.admin.config.title", List.of(
            action(20, "COMPASS", "menu.admin.doctor", "lkjmc doctor", p.status()),
            action(22, "REDSTONE", "menu.admin.reload", "lkjmc config reload", p.reload()),
            input(24, "CLOCK", "menu.admin.restart-warn", "menu.admin.input.restart-warn", "lkjmc restart warn ", p.reload()), back()));
    }

    public static MenuSpec security(AdminMenuPermissions p) {
        return menu("admin-security", "menu.admin.security.title", List.of(
            action(19, "BOOK", "menu.admin.roles", "lkjmc admin role list", p.admin()),
            input(20, "LIME_DYE", "menu.admin.grant", "menu.admin.input.grant", "lkjmc admin grant ", p.admin()),
            input(21, "PLAYER_HEAD", "menu.admin.inspect", "menu.admin.input.inspect", "lkjmc admin inspect ", p.admin()),
            input(22, "RED_DYE", "menu.admin.revoke", "menu.admin.input.revoke", "lkjmc admin revoke ", p.admin()),
            action(23, "NETHER_STAR", "menu.admin.token.status", "lkjmc security daemon-token status", p.admin()),
            action(25, "TNT", "menu.admin.token.rotate", "lkjmc security daemon-token rotate", p.admin()), back()));
    }

    public static MenuSpec economy(AdminMenuPermissions p) {
        return menu("admin-economy", "menu.admin.economy.title", List.of(
            action(20, "EMERALD", "menu.admin.seed-defaults", "lkjmc economy seed-defaults", p.economy()),
            input(22, "CHEST", "menu.admin.shop-upsert", "menu.admin.input.shop-upsert", "lkjmc shop item upsert ", false),
            input(24, "OAK_SIGN", "menu.admin.announce", "menu.admin.input.announce", "announce ", p.announce()), back()));
    }

    public static MenuSpec moderation(AdminMenuPermissions p) {
        return menu("admin-moderation", "menu.admin.moderation.title", List.of(
            action(19, "REDSTONE_TORCH", "menu.reports.title", "reports", p.reports()),
            input(20, "PAPER", "menu.admin.warn", "menu.admin.input.warn", "warn ", p.warn()),
            input(21, "WRITABLE_BOOK", "menu.admin.note", "menu.admin.input.note", "note ", p.warn()),
            input(22, "IRON_AXE", "menu.admin.ban", "menu.admin.input.ban", "ban ", p.ban()),
            input(23, "BARRIER", "menu.admin.mute", "menu.admin.input.mute", "mute ", p.mute()),
            action(24, "GOLDEN_SHOVEL", "menu.admin.claims", "claim list", p.claim()), back()));
    }

    public static MenuSpec audit(AdminMenuPermissions p) {
        return menu("admin-audit", "menu.admin.audit.title", List.of(
            action(22, "PAPER", "menu.admin.audit.tail", "lkjmc admin audit 50", p.admin()), back()));
    }

    public static MenuSpec web(AdminMenuPermissions p) {
        return menu("admin-web", "menu.admin.web.title", List.of(
            action(22, "OAK_SIGN", "menu.admin.web.status", "lkjmc status", p.status(), "menu.admin.web.status.lore"), back()));
    }

    private static SlotSpec action(int slot, String material, String key, String command, boolean enabled, String... lore) {
        return slot(slot, material, key, enabled ? new MenuAction.RunPlayerCommand(command) : denied(),
            enabled ? ItemVisualRole.ACTION : ItemVisualRole.DISABLED, lore);
    }

    private static SlotSpec input(int slot, String material, String key, String prompt, String prefix, boolean enabled) {
        return slot(slot, material, key, enabled ? new MenuAction.TextInput(prompt, prefix) : denied(),
            enabled ? ItemVisualRole.ACTION : ItemVisualRole.DISABLED);
    }

    private static SlotSpec open(int slot, String material, String key, String id, String... lore) {
        return slot(slot, material, key, new MenuAction.OpenRoute(new MenuRoute(new MenuId(id))), ItemVisualRole.NAVIGATION, lore);
    }

    private static SlotSpec back() { return MenuChrome.back(); }
    private static MenuAction denied() { return new MenuAction.Disabled("menu.disabled.admin-permission"); }

    private static MenuSpec menu(String id, String title, List<SlotSpec> functional) {
        var slots = new TreeMap<Integer, SlotSpec>();
        for (var slot : functional) { slots.put(slot.slot(), slot); }
        MenuChrome.applyBorder(slots, MenuTheme.SETTINGS);
        return new MenuSpec(new MenuId(id), new MenuTitle(title), new MenuSize(54), new ArrayList<>(slots.values()));
    }

    private static SlotSpec slot(int slot, String material, String key, MenuAction action,
                                 ItemVisualRole role, String... lore) {
        return new SlotSpec(slot, new ItemSpec(material, key, List.of(lore), role), action);
    }

    private static String title(String id) {
        return switch (id) {
            case "admin-servers" -> "menu.admin.servers.title";
            case "admin-server-detail" -> "menu.server-detail.title";
            case "admin-server-stop-confirm", "admin-server-restart-confirm", "admin-server-delete-confirm" -> "menu.confirm.yes";
            case "admin-server-create-kind", "admin-server-create-template", "admin-server-create-confirm" -> "menu.admin.server.create";
            case "admin-config" -> "menu.admin.config.title";
            case "admin-security" -> "menu.admin.security.title";
            case "admin-economy" -> "menu.admin.economy.title";
            case "admin-moderation" -> "menu.admin.moderation.title";
            case "admin-audit" -> "menu.admin.audit.title";
            case "admin-web" -> "menu.admin.web.title";
            default -> "menu.admin.title";
        };
    }
}
