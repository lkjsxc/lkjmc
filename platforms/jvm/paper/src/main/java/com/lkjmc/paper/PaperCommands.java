package com.lkjmc.paper;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.permission.PermissionNodes;
import java.util.Map;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class PaperCommands implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MenuInventoryAdapter menus;
    private final MessageRenderer renderer;
    private final LanguageCommandAdapter languages;
    private final PaperAdminCommandAdapter admin;
    private final PointsCommandAdapter points;
    private final HomeCommandAdapter homes;
    private final WarpCommandAdapter warps;
    private final TeleportCommandAdapter teleports;
    private final PartyCommandAdapter parties;
    private final AchievementCommandAdapter achievements;
    private final HudCommandAdapter hud;
    private final ShopCommandAdapter shop;

    public PaperCommands(LkjmcPaperPlugin plugin, MenuInventoryAdapter menus, MessageCatalog catalog, LocaleResolver resolver) {
        this.plugin = plugin;
        this.menus = menus;
        this.renderer = new MessageRenderer(catalog, resolver);
        this.languages = new LanguageCommandAdapter(plugin, renderer);
        this.admin = new PaperAdminCommandAdapter(plugin);
        this.points = new PointsCommandAdapter(plugin, renderer);
        this.homes = new HomeCommandAdapter(plugin, renderer);
        this.warps = new WarpCommandAdapter(plugin, renderer);
        this.teleports = new TeleportCommandAdapter(plugin, renderer);
        this.parties = new PartyCommandAdapter(plugin, renderer);
        this.achievements = new AchievementCommandAdapter(plugin, renderer);
        this.hud = new HudCommandAdapter(plugin, renderer);
        this.shop = new ShopCommandAdapter(plugin, renderer);
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (label.equalsIgnoreCase("menu")) {
            return openMenu(sender);
        }
        if (label.equalsIgnoreCase("lang")) {
            return setLanguage(sender, args);
        }
        if (label.equalsIgnoreCase("points")) {
            return showPoints(sender, args);
        }
        if (label.equalsIgnoreCase("sethome")) {
            return homeCommand(sender, args, true);
        }
        if (label.equalsIgnoreCase("home")) {
            return homeCommand(sender, args, false);
        }
        if (label.equalsIgnoreCase("setwarp")) {
            return warpCommand(sender, args, true);
        }
        if (label.equalsIgnoreCase("warp")) {
            return warpCommand(sender, args, false);
        }
        if (label.equalsIgnoreCase("tpa")) {
            return teleportCommand(sender, args, true);
        }
        if (label.equalsIgnoreCase("tpaccept")) {
            return teleportCommand(sender, args, false);
        }
        if (label.equalsIgnoreCase("party")) {
            return partyCommand(sender, args);
        }
        if (label.equalsIgnoreCase("achievements")) {
            return achievementsCommand(sender);
        }
        if (label.equalsIgnoreCase("hud")) {
            return hudCommand(sender, args);
        }
        if (label.equalsIgnoreCase("shop")) {
            return shopCommand(sender, true, args);
        }
        if (label.equalsIgnoreCase("buy")) {
            return shopCommand(sender, false, args);
        }
        return admin.handle(sender, args);
    }
    private boolean openMenu(CommandSender sender) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        plugin.scheduler().runPlayer(player, () -> menus.openRoot(player));
        return true;
    }
    private boolean shopCommand(CommandSender sender, boolean list, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (!player.hasPermission(PermissionNodes.USER_SHOP)) {
            player.sendMessage(message(player, "command.no-permission"));
            return true;
        }
        return list ? shop.list(player) : shop.buy(player, args);
    }
    private boolean hudCommand(CommandSender sender, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        return hud.set(player, args);
    }
    private boolean achievementsCommand(CommandSender sender) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        return achievements.list(player);
    }
    private boolean partyCommand(CommandSender sender, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (!player.hasPermission(PermissionNodes.USER_PARTY)) {
            player.sendMessage(message(player, "command.no-permission"));
            return true;
        }
        return parties.handle(player, args);
    }
    private boolean teleportCommand(CommandSender sender, String[] args, boolean request) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (!player.hasPermission(PermissionNodes.USER_TELEPORT_REQUEST)) {
            player.sendMessage(message(player, "command.no-permission"));
            return true;
        }
        return request ? teleports.request(player, args) : teleports.accept(player, args);
    }

    private boolean warpCommand(CommandSender sender, String[] args, boolean set) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        var node = set ? PermissionNodes.ADMIN_WARP : PermissionNodes.USER_WARP;
        if (!player.hasPermission(node)) {
            player.sendMessage(message(player, "command.no-permission"));
            return true;
        }
        return set ? warps.setWarp(player, args) : warps.warp(player, args);
    }

    private boolean homeCommand(CommandSender sender, String[] args, boolean set) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (!player.hasPermission(PermissionNodes.USER_HOME)) {
            player.sendMessage(message(player, "command.no-permission"));
            return true;
        }
        return set ? homes.setHome(player, args) : homes.home(player, args);
    }

    private boolean showPoints(CommandSender sender, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (!player.hasPermission(PermissionNodes.USER_POINTS)) {
            player.sendMessage(message(player, "command.no-permission"));
            return true;
        }
        return points.show(player, args);
    }

    private boolean setLanguage(CommandSender sender, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (!player.hasPermission(PermissionNodes.USER_LANGUAGE)) {
            player.sendMessage(message(player, "command.no-permission"));
            return true;
        }
        return languages.set(player, args);
    }

    private String message(Player player, String key) {
        return renderer.render(player.locale().toLanguageTag(), key, Map.of("usage", "/lang <en|ja>"));
    }

}
