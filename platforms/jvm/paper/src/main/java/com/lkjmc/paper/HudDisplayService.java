package com.lkjmc.paper;

import com.lkjmc.common.actionbar.ActionBarFrame;
import com.lkjmc.common.actionbar.ActionBarReducer;
import com.lkjmc.common.actionbar.ActionBarState;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.time.Duration;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import net.kyori.adventure.text.Component;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;

public final class HudDisplayService implements Listener {
    private final LkjmcPaperPlugin plugin;
    private static final long PASSIVE_REFRESH_MILLIS = Duration.ofSeconds(60).toMillis();
    private final MessageRenderer renderer;
    private final ConcurrentHashMap<UUID, Player> players = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<UUID, ActionBarState> states = new ConcurrentHashMap<>();

    public HudDisplayService(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
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
        if (plugin.daemon().isEmpty()) {
            return;
        }
        for (var player : players.values()) {
            plugin.daemon().get().send(request(player.getUniqueId())).thenAccept(response -> {
                if (response.ok()) {
                    plugin.scheduler().runPlayer(player, () -> render(player,
                        DaemonJson.bool(response.body(), "hudEnabled")));
                }
            });
        }
    }

    private void render(Player player, boolean enabled) {
        var text = renderer.render(player.locale().toLanguageTag(), "hud.enabled", Map.of());
        var now = System.currentTimeMillis();
        var frame = new ActionBarFrame(1, text, "hud-enabled", now + PASSIVE_REFRESH_MILLIS);
        var decision = ActionBarReducer.reduce(now, enabled,
            states.getOrDefault(player.getUniqueId(), ActionBarState.empty()), List.of(frame), PASSIVE_REFRESH_MILLIS);
        states.put(player.getUniqueId(), decision.state());
        decision.frame().ifPresent(value -> player.sendActionBar(Component.text(value.text())));
    }

    private static DaemonRequest request(UUID playerId) {
        return new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId()),
            "player.settings.get",
            Map.of("playerUuid", playerId.toString())
        );
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
