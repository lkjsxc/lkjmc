package com.lkjmc.paper;

import com.lkjmc.common.claim.ClaimChunk;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.permission.PermissionNodes;
import java.util.Map;
import java.util.UUID;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class ClaimCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final ClaimSnapshotService snapshots;

    public ClaimCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer, ClaimSnapshotService snapshots) {
        this.plugin = plugin;
        this.renderer = renderer;
        this.snapshots = snapshots;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (args.length == 2 && args[0].equalsIgnoreCase("create")) {
            return create(player, args[1]);
        }
        if (args.length == 1 && args[0].equalsIgnoreCase("list")) {
            return list(player);
        }
        if (args.length == 2 && args[0].equalsIgnoreCase("delete")) {
            return delete(player, args[1]);
        }
        if (args.length == 2 && args[0].equalsIgnoreCase("trust")) {
            return trust(player, args[1], true);
        }
        if (args.length == 2 && args[0].equalsIgnoreCase("untrust")) {
            return trust(player, args[1], false);
        }
        if (args.length == 1 && args[0].equalsIgnoreCase("here")) {
            return here(player);
        }
        player.sendMessage(message(player, "command.usage", Map.of("usage", "/claim create|list|delete|trust|untrust|here")));
        return true;
    }

    private boolean create(Player player, String name) {
        return send(player, "claim.create", body(player, Map.of("name", name)), response -> {
            snapshots.refresh();
            player.sendMessage(message(player, response.ok() ? "claim.created" : "claim.failed", Map.of()));
        });
    }

    private boolean list(Player player) {
        return send(player, "claim.list", Map.of("ownerUuid", player.getUniqueId().toString()), response -> {
            var count = DaemonJson.arraySize(response.body(), "claims");
            player.sendMessage(message(player, "claim.list.count", Map.of("count", Integer.toString(count))));
        });
    }

    private boolean delete(Player player, String name) {
        return send(player, "claim.delete", Map.of("ownerUuid", player.getUniqueId().toString(), "name", name), response -> {
            snapshots.refresh();
            player.sendMessage(message(player, response.ok() ? "claim.deleted" : "claim.failed", Map.of()));
        });
    }

    private boolean trust(Player player, String targetName, boolean add) {
        var target = plugin.getServer().getPlayerExact(targetName);
        if (target == null) {
            player.sendMessage(message(player, "claim.player-missing", Map.of()));
            return true;
        }
        var extra = Map.<String, Object>of(
            "trustedUuid", target.getUniqueId().toString(), "trustedName", target.getName()
        );
        return send(player, add ? "claim.trust" : "claim.untrust", body(player, extra), response -> {
            snapshots.refresh();
            var key = add ? "claim.trust.added" : "claim.trust.removed";
            player.sendMessage(message(player, response.ok() ? key : "claim.failed", Map.of("player", target.getName())));
        });
    }

    private boolean here(Player player) {
        var chunk = chunk(player);
        var claim = plugin.claims().snapshot().at(chunk);
        if (claim.isEmpty()) {
            player.sendMessage(message(player, "claim.here.none", Map.of()));
            return true;
        }
        player.sendMessage(message(player, "claim.here.owner", Map.of(
            "name", claim.get().name(), "owner", claim.get().ownerName()
        )));
        return true;
    }

    private boolean send(Player player, String command, Map<String, Object> body,
        java.util.function.Consumer<com.lkjmc.common.daemon.DaemonResponse> handler) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> handler.accept(response))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private Map<String, Object> body(Player player, Map<String, Object> extra) {
        var chunk = chunk(player);
        var body = new java.util.HashMap<String, Object>(extra);
        body.put("ownerUuid", player.getUniqueId().toString());
        body.put("ownerName", player.getName());
        body.put("instanceId", chunk.instanceId());
        body.put("worldName", chunk.worldName());
        body.put("chunkX", chunk.chunkX());
        body.put("chunkZ", chunk.chunkZ());
        body.put("operator", player.hasPermission(PermissionNodes.ADMIN_CLAIM));
        return body;
    }

    private static ClaimChunk chunk(Player player) {
        var chunk = player.getLocation().getChunk();
        return new ClaimChunk(instanceId(), player.getWorld().getName(), chunk.getX(), chunk.getZ());
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
