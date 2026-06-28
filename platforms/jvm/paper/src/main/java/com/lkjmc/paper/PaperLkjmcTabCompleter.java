package com.lkjmc.paper;

import com.lkjmc.common.command.CommandCompletionContext;
import com.lkjmc.common.command.CommandPlatform;
import com.lkjmc.common.command.LkjmcCommandTree;
import java.util.List;
import java.util.function.Supplier;
import org.bukkit.command.Command;
import org.bukkit.command.CommandSender;
import org.bukkit.command.TabCompleter;

final class PaperLkjmcTabCompleter implements TabCompleter {
    private final Supplier<CommandCompletionContext> context;

    PaperLkjmcTabCompleter(LkjmcPaperPlugin plugin) {
        this(() -> context(plugin));
    }

    PaperLkjmcTabCompleter(Supplier<CommandCompletionContext> context) {
        this.context = context;
    }

    @Override
    public List<String> onTabComplete(CommandSender sender, Command command, String alias, String[] args) {
        return LkjmcCommandTree.suggest(CommandPlatform.PAPER, List.of(args), sender::hasPermission, context.get());
    }

    private static CommandCompletionContext context(LkjmcPaperPlugin plugin) {
        var players = plugin.getServer().getOnlinePlayers().stream()
            .map(player -> player.getName())
            .sorted()
            .toList();
        return new CommandCompletionContext(List.of(), players, List.of("paper", "folia", "purpur"));
    }
}
