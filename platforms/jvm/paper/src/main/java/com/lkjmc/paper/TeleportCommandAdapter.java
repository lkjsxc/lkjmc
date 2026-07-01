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
        var target = plugin.getServer().getPlayerExact(args[0]);
        if (target == null) {
            source.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
                ProfileTransferMessages.tpaRequest(args[0]));
            source.sendMessage(message(source, "teleport.request.sent", Map.of("player", args[0])));
            return true;
        }
        if (target.getUniqueId().equals(source.getUniqueId())) {
            source.sendMessage(message(source, "teleport.request.missing", Map.of()));
            return true;
        }
        requests.put(target.getUniqueId(), new Request(source.getUniqueId(), Instant.now().plusSeconds(60)));
        source.sendMessage(message(source, "teleport.request.sent", Map.of("player", target.getName())));
        target.sendMessage(message(target, "teleport.request.received", Map.of("player", source.getName())));
        return true;
    }

    public boolean accept(Player target, String[] args) {
        var request = Optional.ofNullable(requests.remove(target.getUniqueId()))
            .filter(value -> value.expiresAt().isAfter(Instant.now()));
        if (request.isEmpty()) {
            if (args.length == 1) {
                target.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
                    ProfileTransferMessages.tpaAccept(args[0], location(target.getLocation())));
                return true;
            }
            target.sendMessage(message(target, "teleport.request.none", Map.of()));
            return true;
        }
        var source = plugin.getServer().getPlayer(request.get().source());
        if (source == null || (args.length == 1 && !source.getName().equalsIgnoreCase(args[0]))) {
            target.sendMessage(message(target, "teleport.request.missing", Map.of()));
            return true;
        }
        plugin.scheduler().runPlayer(source, () -> {
            source.teleport(target.getLocation());
            source.sendMessage(message(source, "teleport.request.accepted", Map.of("player", target.getName())));
        });
        target.sendMessage(message(target, "teleport.request.accepted", Map.of("player", source.getName())));
        return true;
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }

    private static String location(Location location) {
        return String.join("|",
            location.getWorld().getName(),
            Double.toString(location.getX()),
            Double.toString(location.getY()),
            Double.toString(location.getZ()),
            Float.toString(location.getYaw()),
            Float.toString(location.getPitch())
        );
    }

    private record Request(UUID source, Instant expiresAt) {}
}
