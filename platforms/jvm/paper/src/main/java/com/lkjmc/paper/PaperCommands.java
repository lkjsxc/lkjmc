package com.lkjmc.paper;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class PaperCommands implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MenuInventoryAdapter menus;

    public PaperCommands(LkjmcPaperPlugin plugin, MenuInventoryAdapter menus) {
        this.plugin = plugin;
        this.menus = menus;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (label.equalsIgnoreCase("menu")) {
            return openMenu(sender);
        }
        if (args.length == 1 && args[0].equalsIgnoreCase("status")) {
            sender.sendMessage("lkjmc paper running; players=" + plugin.getServer().getOnlinePlayers().size());
            return true;
        }
        sender.sendMessage("usage: /lkjmc status");
        return true;
    }

    private boolean openMenu(CommandSender sender) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        plugin.scheduler().runPlayer(player, () -> menus.openRoot(player));
        return true;
    }
}
