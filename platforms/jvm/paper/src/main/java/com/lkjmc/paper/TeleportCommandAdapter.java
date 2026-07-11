package com.lkjmc.paper;

import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.time.Instant;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import org.bukkit.Location;
import org.bukkit.entity.Player;

public final class TeleportCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final Map<UUID, Request> requests = new ConcurrentHashMap<>();

    public TeleportCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public boolean request(Player source, String[] args) {
        if (args.length != 1) {
            source.sendMessage(message(source, "command.usage", Map.of("usage", "/tpa <player>")));
            return true;
        }
        var sourceId = source.getUniqueId();
        var sourceName = source.getName();
        plugin.scheduler().runGlobal(() -> resolveRequest(source, sourceId, sourceName, args[0]));
        return true;
    }

    public boolean accept(Player target, String[] args) {
        var request = Optional.ofNullable(requests.remove(target.getUniqueId()))
            .filter(value -> value.expiresAt().isAfter(Instant.now()));
        if (request.isEmpty()) {
            if (args.length == 1) target.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
                ProfileTransferMessages.tpaAccept(args[0], location(target.getLocation())));
            else target.sendMessage(message(target, "teleport.request.none", Map.of()));
            return true;
        }
        var targetLocation = target.getLocation();
        var targetName = target.getName();
        var requestedName = args.length == 1 ? args[0] : "";
        plugin.scheduler().runGlobal(() -> resolveAccept(target, targetName, targetLocation, request.get(), requestedName));
        return true;
    }

    private void resolveRequest(Player source, UUID sourceId, String sourceName, String targetName) {
        var target = plugin.getServer().getPlayerExact(targetName);
        if (target == null) {
            plugin.scheduler().runPlayer(source, () -> {
                source.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL, ProfileTransferMessages.tpaRequest(targetName));
                source.sendMessage(message(source, "teleport.request.sent", Map.of("player", targetName)));
            });
            return;
        }
        plugin.scheduler().runPlayer(target, () -> {
            if (target.getUniqueId().equals(sourceId)) {
                plugin.scheduler().runPlayer(source, () -> source.sendMessage(message(source, "teleport.request.missing", Map.of())));
                return;
            }
            requests.put(target.getUniqueId(), new Request(sourceId, Instant.now().plusSeconds(60)));
            target.sendMessage(message(target, "teleport.request.received", Map.of("player", sourceName)));
            plugin.scheduler().runPlayer(source, () -> source.sendMessage(message(source,
                "teleport.request.sent", Map.of("player", target.getName()))));
        });
    }

    private void resolveAccept(Player target, String targetName, Location targetLocation,
                               Request request, String requestedName) {
        var source = plugin.getServer().getPlayer(request.source());
        if (source == null) {
            plugin.scheduler().runPlayer(target, () -> target.sendMessage(message(target, "teleport.request.missing", Map.of())));
            return;
        }
        plugin.scheduler().runPlayer(source, () -> {
            if (!requestedName.isBlank() && !source.getName().equalsIgnoreCase(requestedName)) {
                plugin.scheduler().runPlayer(target, () -> target.sendMessage(message(target, "teleport.request.missing", Map.of())));
                return;
            }
            var sourceName = source.getName();
            source.teleportAsync(targetLocation).whenComplete((ok, error) ->
                completeAccept(source, target, sourceName, targetName, error == null && Boolean.TRUE.equals(ok)));
        });
    }

    private void completeAccept(Player source, Player target, String sourceName, String targetName, boolean accepted) {
        var key = accepted ? "teleport.request.accepted" : "teleport.request.missing";
        plugin.scheduler().runPlayer(source, () -> source.sendMessage(message(source, key, Map.of("player", targetName))));
        plugin.scheduler().runPlayer(target, () -> target.sendMessage(message(target, key, Map.of("player", sourceName))));
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }

    private static String location(Location location) {
        return String.join("|", location.getWorld().getName(), Double.toString(location.getX()),
            Double.toString(location.getY()), Double.toString(location.getZ()), Float.toString(location.getYaw()),
            Float.toString(location.getPitch()));
    }

    private record Request(UUID source, Instant expiresAt) {}
}
