package com.lkjmc.paper;

import com.lkjmc.common.ui.kernel.MenuRoute;
import com.lkjmc.paper.ui.UiEntrypoints;
import java.util.Arrays;
import java.util.Map;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class DocsCommandAdapter implements CommandExecutor {
    private final UiEntrypoints entrypoints;

    public DocsCommandAdapter(UiEntrypoints entrypoints) {
        this.entrypoints = entrypoints;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        entrypoints.openDeep(player, route(args == null ? new String[0] : args));
        return true;
    }

    private MenuRoute route(String[] args) {
        if (args.length > 1 && args[0].equalsIgnoreCase("search")) {
            var query = String.join(" ", Arrays.copyOfRange(args, 1, args.length));
            return new MenuRoute("docs-search", Map.of("query", query));
        }
        if (args.length == 1 && !args[0].isBlank()) {
            return new MenuRoute("docs-file", Map.of("path", args[0], "page", "0"));
        }
        return new MenuRoute("docs-directory", Map.of("path", "docs"));
    }
}
