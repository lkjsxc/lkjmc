package com.lkjmc.paper;

import java.util.Arrays;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class DocsCommandAdapter implements CommandExecutor {
    private final PaperMenuAdapter docs;

    public DocsCommandAdapter(PaperMenuAdapter docs) {
        this.docs = docs;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (label.equalsIgnoreCase("menu")) {
            docs.openRoot(player);
        } else if (args.length == 0) {
            docs.openDocs(player);
        } else if (args.length > 1 && args[0].equalsIgnoreCase("search")) {
            docs.openSearch(player, String.join(" ", Arrays.copyOfRange(args, 1, args.length)));
        } else {
            docs.openPath(player, String.join(" ", args));
        }
        return true;
    }
}
