package com.lkjmc.paper;

import com.lkjmc.common.command.CommandCompletionContext;
import com.lkjmc.common.command.CommandPlatform;
import com.lkjmc.common.command.LkjmcCommandTree;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.permission.PermissionSnapshotCache;
import com.lkjmc.common.permission.PrincipalIdentity;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Supplier;
import org.bukkit.command.Command;
import org.bukkit.command.CommandSender;
import org.bukkit.command.TabCompleter;

final class PaperLkjmcTabCompleter implements TabCompleter {
    private final LkjmcPaperPlugin plugin;
    private final Supplier<CommandCompletionContext> testContext;
    private final PermissionSnapshotCache adminGrants;
    private final AtomicBoolean refreshing = new AtomicBoolean(false);
    private volatile List<String> serverIds = List.of();

    PaperLkjmcTabCompleter(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
        this.testContext = null;
        this.adminGrants = plugin == null ? PermissionSnapshotCache.disabled() : plugin.adminGrants();
    }

    PaperLkjmcTabCompleter(Supplier<CommandCompletionContext> context) {
        this(context, PermissionSnapshotCache.disabled());
    }

    PaperLkjmcTabCompleter(Supplier<CommandCompletionContext> context, PermissionSnapshotCache adminGrants) {
        this.plugin = null;
        this.testContext = context;
        this.adminGrants = adminGrants == null ? PermissionSnapshotCache.disabled() : adminGrants;
    }

    @Override
    public List<String> onTabComplete(CommandSender sender, Command command, String alias, String[] args) {
        return LkjmcCommandTree.suggest(CommandPlatform.PAPER, List.of(args),
            permission -> allowed(sender, permission), context());
    }

    private boolean allowed(CommandSender sender, String permission) {
        var platform = sender.hasPermission(permission);
        if (sender instanceof org.bukkit.entity.Player player) {
            platform = platform || player.isOp();
            return adminGrants.decide(identity(player), permission, platform, player.isOp()).allowed();
        }
        return platform;
    }

    private PrincipalIdentity identity(org.bukkit.entity.Player player) {
        return new PrincipalIdentity("minecraft-player", player.getUniqueId().toString(), player.getName());
    }

    private CommandCompletionContext context() {
        if (testContext != null) {
            return testContext.get();
        }
        refreshServerIds();
        var players = plugin.getServer().getOnlinePlayers().stream()
            .map(player -> player.getName())
            .sorted()
            .toList();
        return new CommandCompletionContext(serverIds, players, List.of("paper", "folia", "purpur"),
            List.of("owner", "operator", "moderator", "support", "builder"),
            List.of("end-expedition"),
            List.of(), List.of(), List.of(), players);
    }

    private void refreshServerIds() {
        if (plugin.daemon().isEmpty() || !refreshing.compareAndSet(false, true)) {
            return;
        }
        var request = new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", "completion"),
            "instance.list", Map.of());
        plugin.daemon().get().send(request).whenComplete((response, error) -> {
            try {
                if (error == null && response != null && response.ok() && response.body().has("instances")) {
                    serverIds = parseServerIds(response.body().getAsJsonArray("instances")).stream().sorted().toList();
                }
            } finally {
                refreshing.set(false);
            }
        });
    }

    private static List<String> parseServerIds(com.google.gson.JsonArray instances) {
        var ids = new ArrayList<String>();
        for (var value : instances) {
            if (value.isJsonObject() && value.getAsJsonObject().has("id")) {
                ids.add(value.getAsJsonObject().get("id").getAsString());
            }
        }
        return ids;
    }
}
