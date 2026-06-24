package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import java.util.Map;
import java.util.UUID;
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
}
