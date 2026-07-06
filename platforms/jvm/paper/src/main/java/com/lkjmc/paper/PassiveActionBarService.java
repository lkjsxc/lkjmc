package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.actionbar.ActionBarFormatter;
import com.lkjmc.common.actionbar.ActionBarFrame;
import com.lkjmc.common.actionbar.ActionBarReducer;
import com.lkjmc.common.actionbar.ActionBarSnapshot;
import com.lkjmc.common.actionbar.ActionBarState;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.i18n.MiniMessageText;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;

public final class PassiveActionBarService implements Listener {
    private static final long REFRESH_MILLIS = 0;
    private static final long FRAME_TTL_MILLIS = Duration.ofSeconds(2).toMillis();
    private static final long STALE_MILLIS = Duration.ofSeconds(15).toMillis();
    private final LkjmcPaperPlugin plugin;
    private final MiniMessageText text;
    private final ConcurrentHashMap<UUID, Player> players = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<UUID, ActionBarState> states = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<UUID, CachedSnapshot> snapshots = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<UUID, Long> joinedAt = new ConcurrentHashMap<>();

    public PassiveActionBarService(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.text = new MiniMessageText(renderer.catalog(), renderer.resolver());
    }

    public void start() {
        plugin.scheduler().runAsyncRepeating(this::snapshotTick, Duration.ZERO, Duration.ofSeconds(1));
        plugin.scheduler().runAsyncRepeating(this::renderTick, Duration.ofMillis(200), Duration.ofMillis(200));
    }

    @EventHandler
    public void onJoin(PlayerJoinEvent event) {
        var player = event.getPlayer();
        players.put(player.getUniqueId(), player);
        joinedAt.put(player.getUniqueId(), System.currentTimeMillis());
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        var id = event.getPlayer().getUniqueId();
        players.remove(id);
        states.remove(id);
        snapshots.remove(id);
        joinedAt.remove(id);
    }

    private void snapshotTick() {
        plugin.daemon().ifPresent(client -> players.values().forEach(player ->
            client.send(request(player)).thenAccept(response -> {
                if (response.ok()) {
                    snapshots.put(player.getUniqueId(), new CachedSnapshot(snapshot(response.body()), System.currentTimeMillis()));
                }
            })));
    }

    private void renderTick() {
        players.values().forEach(player -> plugin.scheduler().runPlayer(player,
            () -> render(player, currentSnapshot(player))));
    }

    private ActionBarSnapshot currentSnapshot(Player player) {
        var cached = snapshots.get(player.getUniqueId());
        if (cached != null && System.currentTimeMillis() - cached.loadedAtMillis() <= STALE_MILLIS) {
            return cached.snapshot();
        }
        return localSnapshot(player);
    }

    private ActionBarSnapshot localSnapshot(Player player) {
        var joined = joinedAt.getOrDefault(player.getUniqueId(), System.currentTimeMillis());
        var playtime = Math.max(0, (System.currentTimeMillis() - joined) / 1000);
        var online = plugin.getServer().getOnlinePlayers().size();
        return new ActionBarSnapshot(true, playtime, -1, instanceId(), online, online, false, 0);
    }

    private void render(Player player, ActionBarSnapshot snapshot) {
        var now = System.currentTimeMillis();
        var frames = frames(player, snapshot, now);
        var current = states.getOrDefault(player.getUniqueId(), ActionBarState.empty());
        var decision = ActionBarReducer.reduce(now, snapshot.hudEnabled(), current, frames, REFRESH_MILLIS);
        states.put(player.getUniqueId(), decision.state());
        decision.frame().ifPresent(value -> player.sendActionBar(MiniMessageText.parseStrict(value.text())));
    }

    private java.util.List<ActionBarFrame> frames(Player player, ActionBarSnapshot snapshot, long now) {
        var frames = new ArrayList<ActionBarFrame>();
        if (snapshot.dailyAvailable()) {
            frames.add(new ActionBarFrame(5, message(player, "actionbar.frame.daily.ready", Map.of()), "daily", now + FRAME_TTL_MILLIS));
        }
        if (snapshot.randomTeleportCooldownSeconds() > 0) {
            frames.add(new ActionBarFrame(4, message(player, "actionbar.frame.rtp.cooldown",
                Map.of("seconds", Long.toString(snapshot.randomTeleportCooldownSeconds()))), "rtp", now + FRAME_TTL_MILLIS));
        }
        frames.add(new ActionBarFrame(1, passive(player, snapshot), "passive:" + snapshot.serverId()
            + ":" + snapshot.balance(), now + FRAME_TTL_MILLIS));
        return java.util.List.copyOf(frames);
    }

    private String passive(Player player, ActionBarSnapshot snapshot) {
        var values = new java.util.HashMap<String, String>();
        values.put("playtime", ActionBarFormatter.playtime(snapshot.playtimeSeconds()));
        values.put("server", snapshot.serverId());
        values.put("serverOnline", Long.toString(snapshot.serverPlayerCount()));
        values.put("networkOnline", Long.toString(snapshot.networkOnlineCount()));
        if (snapshot.balance() >= 0) {
            values.put("points", Long.toString(snapshot.balance()));
            return message(player, "actionbar.frame.passive", values);
        }
        return message(player, "actionbar.frame.passive.no-points", values);
    }

    private String message(Player player, String key, Map<String, String> values) {
        return text.renderMarkup(plugin.localeService().locale(player), key, values);
    }

    private static ActionBarSnapshot snapshot(JsonObject body) {
        return new ActionBarSnapshot(DaemonJson.bool(body, "hudEnabled"), integer(body, "playtimeSeconds"),
            integer(body, "balance", -1), DaemonJson.string(body, "serverId").orElse(instanceId()),
            integer(body, "serverPlayerCount"), integer(body, "networkOnlineCount"),
            DaemonJson.bool(body, "dailyAvailable"), integer(body, "randomTeleportCooldownSeconds"));
    }

    private static DaemonRequest request(Player player) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()),
            "player.actionbar.snapshot", Map.of("playerUuid", player.getUniqueId().toString(), "serverId", instanceId()));
    }

    private static long integer(JsonObject object, String key) { return integer(object, key, 0); }
    private static long integer(JsonObject object, String key, long fallback) {
        return DaemonJson.integer(object, key).orElse(fallback);
    }
    private static String instanceId() { return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper"); }
    private record CachedSnapshot(ActionBarSnapshot snapshot, long loadedAtMillis) {}
}
