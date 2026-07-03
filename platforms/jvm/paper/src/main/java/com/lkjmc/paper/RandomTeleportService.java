package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.menu.RandomTeleportQuote;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import net.kyori.adventure.text.Component;
import org.bukkit.Location;
import org.bukkit.World;
import org.bukkit.entity.Player;

final class RandomTeleportService {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final RandomTeleportSearch search;

    RandomTeleportService(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
        this.search = new RandomTeleportSearch(plugin);
    }

    boolean start(Player player, String profileId, boolean confirmed) {
        daemon().ifPresentOrElse(client -> client.send(daemon("player.random-teleport.quote", Map.of(
            "playerUuid", player.getUniqueId().toString(), "serverId", instanceId(), "profileId", profile(profileId)
        ))).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> handleQuote(player, quote(response.body()), response.ok(), confirmed))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    boolean start(Player player) { return start(player, "overworld", false); }

    void portalBlocked(Player player) {
        portalMessage(player, "rtp nether", "rtp end");
    }

    private void portalMessage(Player player, String nether, String end) {
        var text = message(player, "rtp.portal-disabled", Map.of("nether", nether, "end", end));
        player.sendActionBar(Component.text(text));
        player.sendMessage(text);
    }

    private void handleQuote(Player player, RandomTeleportQuote quote, boolean ok, boolean confirmed) {
        if (!ok || !quote.enabled()) {
            player.sendMessage(message(player, "rtp.failed", Map.of()));
            return;
        }
        if (quote.cooldownRemainingSeconds() > 0) {
            player.sendMessage(message(player, "rtp.cooldown", Map.of("seconds", Long.toString(quote.cooldownRemainingSeconds()))));
            return;
        }
        if (!quote.canAfford()) {
            player.sendMessage(message(player, "rtp.insufficient", Map.of(
                "cost", Long.toString(quote.costPoints()), "balance", Long.toString(quote.balance()))));
            return;
        }
        if (quote.confirmationRequired() && !confirmed) {
            player.sendMessage(message(player, "rtp.quote", Map.of(
                "profile", quote.profileId(), "cost", Long.toString(quote.costPoints()),
                "min", Integer.toString(quote.minRadius()), "max", Integer.toString(quote.maxRadius()),
                "cooldown", Long.toString(quote.cooldownRemainingSeconds()))));
            return;
        }
        player.sendMessage(message(player, "rtp.searching", Map.of()));
        search.find(player, quote, result -> plugin.scheduler().runPlayer(player, () -> {
            if (result.isEmpty()) {
                player.sendMessage(message(player, "rtp.no-safe-location", Map.of()));
            } else {
                reserve(player, quote, result.get());
            }
        }));
    }

    private void reserve(Player player, RandomTeleportQuote quote, Location target) {
        var correlationId = UUID.randomUUID();
        daemon().ifPresentOrElse(client -> client.send(daemon("player.random-teleport.reserve", Map.of(
            "playerUuid", player.getUniqueId().toString(), "name", player.getName(), "serverId", instanceId(),
            "profileId", quote.profileId(), "world", target.getWorld().getName(), "x", target.getX(),
            "y", target.getY(), "z", target.getZ(), "correlationId", correlationId.toString()
        ))).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> {
            if (!response.ok()) {
                player.sendMessage(failure(player, response.error().map(error -> error.code()).orElse("rtp.failed"), quote));
                return;
            }
            teleport(player, target, correlationId);
        })), () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
    }

    private void teleport(Player player, Location target, UUID correlationId) {
        player.teleportAsync(target).whenComplete((ok, error) -> plugin.scheduler().runPlayer(player, () -> {
            if (error == null && Boolean.TRUE.equals(ok)) {
                send("player.random-teleport.complete", player, correlationId, "");
                player.sendActionBar(Component.text(message(player, "rtp.teleported", Map.of())));
                player.sendMessage(message(player, "rtp.teleported", Map.of()));
            } else {
                send("player.random-teleport.refund", player, correlationId, "teleport-failed");
                player.sendMessage(message(player, "rtp.teleport-failed-refunded", Map.of()));
            }
        }));
    }

    private void send(String command, Player player, UUID correlationId, String reason) {
        var body = reason.isBlank() ? Map.<String, Object>of(
            "playerUuid", player.getUniqueId().toString(), "correlationId", correlationId.toString())
            : Map.<String, Object>of("playerUuid", player.getUniqueId().toString(),
                "correlationId", correlationId.toString(), "reason", reason);
        daemon().ifPresent(client -> client.send(daemon(command, body)));
    }

    private String failure(Player player, String code, RandomTeleportQuote quote) {
        if (code.equals("rtp.insufficient_points")) {
            return message(player, "rtp.insufficient", Map.of(
                "cost", Long.toString(quote.costPoints()), "balance", Long.toString(quote.balance())));
        }
        if (code.equals("rtp.cooldown")) {
            return message(player, "rtp.cooldown", Map.of("seconds", Long.toString(quote.cooldownRemainingSeconds())));
        }
        return message(player, "rtp.failed", Map.of());
    }

    private RandomTeleportQuote quote(JsonObject body) {
        return new RandomTeleportQuote(text(body, "profileId", "overworld"), text(body, "targetEnvironment", "normal"),
            bool(body, "confirmationRequired"), bool(body, "enabled"), bool(body, "canAfford"), integer(body, "costPoints"),
            integer(body, "balance"), integer(body, "cooldownRemainingSeconds"), (int) integer(body, "minRadius"),
            (int) integer(body, "maxRadius"), (int) integer(body, "maxAttempts"));
    }

    private DaemonRequest daemon(String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body);
    }

    private Optional<DaemonClient> daemon() { return plugin.daemon(); }
    private static long integer(JsonObject object, String key) { return DaemonJson.integer(object, key).orElse(0L); }
    private static boolean bool(JsonObject object, String key) { return DaemonJson.bool(object, key); }
    private static String text(JsonObject object, String key, String fallback) {
        return DaemonJson.string(object, key).orElse(fallback);
    }
    private static String profile(String profileId) { return profileId == null || profileId.isBlank() ? "overworld" : profileId; }
    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }
    private static String instanceId() { return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper"); }
}
