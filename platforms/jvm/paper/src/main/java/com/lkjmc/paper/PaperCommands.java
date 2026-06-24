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

    public PaperCommands(LkjmcPaperPlugin plugin, MenuInventoryAdapter menus, MessageCatalog catalog, LocaleResolver resolver) {
        this.plugin = plugin;
        this.menus = menus;
        this.renderer = new MessageRenderer(catalog, resolver);
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (label.equalsIgnoreCase("menu")) {
            return openMenu(sender);
        }
        if (label.equalsIgnoreCase("lang")) {
            return setLanguage(sender, args);
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
