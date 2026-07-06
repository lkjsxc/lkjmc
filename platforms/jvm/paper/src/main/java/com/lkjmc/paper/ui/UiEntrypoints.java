package com.lkjmc.paper.ui;

import com.lkjmc.common.ui.kernel.MenuRoute;
import com.lkjmc.common.ui.kernel.UiMsg;
import com.lkjmc.paper.SchedulerBridge;
import java.util.Arrays;
import java.util.Map;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class UiEntrypoints implements CommandExecutor {
    private final SchedulerBridge scheduler;
    private final UiSessionService sessions;

    public UiEntrypoints(SchedulerBridge scheduler, UiSessionService sessions) {
        this.scheduler = scheduler;
        this.sessions = sessions;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (label.equalsIgnoreCase("docs")) {
            openDocs(player, args == null ? new String[0] : args);
            return true;
        }
        openMenu(player);
        return true;
    }

    public void openMenu(Player player) {
        scheduler.runPlayer(player, () -> sessions.openRoot(player));
    }

    public void openHotbar(Player player) {
        openMenu(player);
    }

    public void openDocs(Player player, String[] args) {
        scheduler.runPlayer(player, () -> sessions.dispatch(player, new UiMsg.Open(docsRoute(args))));
    }

    private MenuRoute docsRoute(String[] args) {
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
