package com.lkjmc.paper;

import com.lkjmc.common.actionbar.ActionBarFormatter;
import com.lkjmc.common.actionbar.ActionBarFrame;
import com.lkjmc.common.actionbar.ActionBarReducer;
import com.lkjmc.common.actionbar.ActionBarSnapshot;
import com.lkjmc.common.actionbar.ActionBarState;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import net.kyori.adventure.text.Component;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;

public final class PassiveActionBarService implements Listener {
    private static final long REFRESH_MILLIS = Duration.ofSeconds(60).toMillis();
    private static final long FRAME_TTL_MILLIS = Duration.ofSeconds(8).toMillis();
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final ConcurrentHashMap<UUID, Player> players = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<UUID, ActionBarState> states = new ConcurrentHashMap<>();

    public PassiveActionBarService(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public void start() {
        plugin.scheduler().runAsyncRepeating(this::tick, Duration.ofSeconds(3), Duration.ofSeconds(5));
    }

    @EventHandler
    public void onJoin(PlayerJoinEvent event) {
        players.put(event.getPlayer().getUniqueId(), event.getPlayer());
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        players.remove(event.getPlayer().getUniqueId());
        states.remove(event.getPlayer().getUniqueId());
    }

    private void tick() {
        plugin.daemon().ifPresent(client -> players.values().forEach(player ->
            client.send(request(player)).thenAccept(response -> {
                if (response.ok()) {
                    plugin.scheduler().runPlayer(player, () -> render(player, snapshot(response.body())));
                }
            })));
    }

    private void render(Player player, ActionBarSnapshot snapshot) {
        var now = System.currentTimeMillis();
        var frames = frames(player, snapshot, now);
        var current = states.getOrDefault(player.getUniqueId(), ActionBarState.empty());
        var decision = ActionBarReducer.reduce(now, snapshot.hudEnabled(), current, frames, REFRESH_MILLIS);
        states.put(player.getUniqueId(), decision.state());
        decision.frame().ifPresent(value -> player.sendActionBar(Component.text(value.text())));
    }

    private java.util.List<ActionBarFrame> frames(Player player, ActionBarSnapshot snapshot, long now) {
        var frames = new ArrayList<ActionBarFrame>();
        if (snapshot.dailyAvailable()) {
            frames.add(new ActionBarFrame(5, message(player, "actionbar.daily.ready", Map.of()), "daily", now + FRAME_TTL_MILLIS));
        }
        if (snapshot.randomTeleportCooldownSeconds() > 0) {
            frames.add(new ActionBarFrame(4, message(player, "actionbar.rtp.cooldown",
                Map.of("seconds", Long.toString(snapshot.randomTeleportCooldownSeconds()))), "rtp", now + FRAME_TTL_MILLIS));
        }
        frames.add(new ActionBarFrame(1, message(player, "actionbar.passive", Map.of(
            "playtime", ActionBarFormatter.playtime(snapshot.playtimeSeconds()),
            "points", Long.toString(snapshot.balance()), "server", snapshot.serverId(),
            "serverOnline", Long.toString(snapshot.serverPlayerCount()),
            "networkOnline", Long.toString(snapshot.networkOnlineCount()))),
            "passive:" + snapshot.serverId() + ":" + snapshot.balance(), now + FRAME_TTL_MILLIS));
        return java.util.List.copyOf(frames);
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }

    private static ActionBarSnapshot snapshot(com.google.gson.JsonObject body) {
        return new ActionBarSnapshot(DaemonJson.bool(body, "hudEnabled"), integer(body, "playtimeSeconds"),
            integer(body, "balance"), DaemonJson.string(body, "serverId").orElse(instanceId()),
            integer(body, "serverPlayerCount"), integer(body, "networkOnlineCount"),
            DaemonJson.bool(body, "dailyAvailable"), integer(body, "randomTeleportCooldownSeconds"));
    }

    private static DaemonRequest request(Player player) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()),
            "player.actionbar.snapshot", Map.of(
                "playerUuid", player.getUniqueId().toString(), "serverId", instanceId()
            ));
    }

    private static long integer(com.google.gson.JsonObject object, String key) {
        return DaemonJson.integer(object, key).orElse(0L);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
