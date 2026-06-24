package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.permission.PermissionNodes;
import java.util.Map;
import java.util.UUID;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class PaperCommands implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MenuInventoryAdapter menus;
    private final MessageRenderer renderer;
    private final PointsCommandAdapter points;
    private final HomeCommandAdapter homes;
    private final WarpCommandAdapter warps;
    private final TeleportCommandAdapter teleports;

    public PaperCommands(LkjmcPaperPlugin plugin, MenuInventoryAdapter menus, MessageCatalog catalog, LocaleResolver resolver) {
        this.plugin = plugin;
        this.menus = menus;
        this.renderer = new MessageRenderer(catalog, resolver);
        this.points = new PointsCommandAdapter(plugin, renderer);
        this.homes = new HomeCommandAdapter(plugin, renderer);
        this.warps = new WarpCommandAdapter(plugin, renderer);
        this.teleports = new TeleportCommandAdapter(plugin, renderer);
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
            return showPoints(sender);
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
        if (args.length == 1 && args[0].equalsIgnoreCase("status")) {
            sendStatus(sender);
            return true;
        }
        sender.sendMessage("usage: /lkjmc status");
        return true;
    }

    private void sendStatus(CommandSender sender) {
        sender.sendMessage("lkjmc paper running; players=" + plugin.getServer().getOnlinePlayers().size());
        plugin.daemon().ifPresent(client -> client.send(new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", "paper"),
            "status",
            Map.of()
        )).thenAccept(response -> sender.sendMessage(response.ok() ? "daemon ok" : "daemon failed")));
    }

    private boolean openMenu(CommandSender sender) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        plugin.scheduler().runPlayer(player, () -> menus.openRoot(player));
        return true;
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

    private boolean showPoints(CommandSender sender) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (!player.hasPermission(PermissionNodes.USER_POINTS)) {
            player.sendMessage(message(player, "command.no-permission"));
            return true;
        }
        return points.show(player);
    }

    private boolean setLanguage(CommandSender sender, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (!player.hasPermission(PermissionNodes.USER_LANGUAGE) || args.length != 1 || !validLanguage(args[0])) {
            player.sendMessage(message(player, "command.usage"));
            return true;
        }
        var instanceId = System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId),
            "player.settings.set",
            Map.of(
                "playerUuid", player.getUniqueId().toString(),
                "name", player.getName(),
                "language", args[0].toLowerCase()
            )
        )).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(message(player, response.ok() ? "language.saved" : "language.failed")))),
            () -> player.sendMessage(message(player, "daemon.unavailable")));
        return true;
    }

    private String message(Player player, String key) {
        return renderer.render(player.locale().toLanguageTag(), key, Map.of("usage", "/lang <en|ja>"));
    }


    private static boolean validLanguage(String value) {
        return value.equalsIgnoreCase("en") || value.equalsIgnoreCase("ja");
    }
}
