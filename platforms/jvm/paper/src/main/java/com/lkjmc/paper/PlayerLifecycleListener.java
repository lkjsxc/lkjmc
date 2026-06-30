package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.permission.PrincipalIdentity;
import java.time.Duration;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;

public final class PlayerLifecycleListener implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final PlayerProfileAdapter profiles = new PlayerProfileAdapter();

    public PlayerLifecycleListener(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
    }

    @EventHandler
    public void onJoin(PlayerJoinEvent event) {
        scheduleGrantRefresh(event.getPlayer());
        var context = context();
        if (context.isEmpty()) {
            return;
        }
        var playerId = event.getPlayer().getUniqueId().toString();
        var request = request(context.get().instanceId(), "player.load", Map.of("playerUuid", playerId, "scope", "profile"));
        context.get().client().send(request).thenAccept(response -> DaemonJson.string(response.body(), "payloadBase64")
            .ifPresent(payload -> apply(event.getPlayer(), payload)));
        recordJoin(context.get(), event.getPlayer());
        grantFirstLogin(context.get(), event.getPlayer());
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        plugin.adminGrants().evict(identity(event.getPlayer()));
        var context = context();
        if (context.isEmpty()) {
            return;
        }
        var snapshot = profiles.capture(event.getPlayer());
        context.get().client().send(request(context.get().instanceId(), "player.snapshot", Map.of(
            "playerUuid", event.getPlayer().getUniqueId().toString(),
            "name", event.getPlayer().getName(),
            "sourceInstance", context.get().instanceId(),
            "scope", "profile",
            "payloadBase64", snapshot.payloadBase64(),
            "sha256", snapshot.sha256()
        )));
        context.get().client().send(request(context.get().instanceId(), "player.session.leave", Map.of(
            "playerUuid", event.getPlayer().getUniqueId().toString(), "serverId", context.get().instanceId()
        )));
    }

    private void apply(Player player, String payloadBase64) {
        plugin.scheduler().runPlayer(player, () -> profiles.apply(player, payloadBase64));
    }

    private void recordJoin(Context context, Player player) {
        context.client().send(request(context.instanceId(), "player.session.join", Map.of(
            "playerUuid", player.getUniqueId().toString(), "name", player.getName(), "serverId", context.instanceId()
        )));
    }

    private void grantFirstLogin(Context context, Player player) {
        context.client().send(request(context.instanceId(), "player.achievement.grant", Map.of(
            "playerUuid", player.getUniqueId().toString(),
            "playerName", player.getName(),
            "achievementId", "first-login",
            "titleKey", "achievement.first-login"
        )));
    }

    private void scheduleGrantRefresh(Player player) {
        if (!plugin.adminGrants().enabled()) {
            return;
        }
        plugin.adminGrants().refresh(identity(player)).exceptionally(error -> null);
        plugin.scheduler().runPlayerLater(player, () -> {
            if (player.isOnline()) {
                scheduleGrantRefresh(player);
            }
        }, Duration.ofSeconds(30));
    }

    private PrincipalIdentity identity(Player player) {
        return new PrincipalIdentity("minecraft-player", player.getUniqueId().toString(), player.getName());
    }

    private Optional<Context> context() {
        var instanceId = System.getenv("LKJMC_INSTANCE_ID");
        if (instanceId == null || instanceId.isBlank() || plugin.daemon().isEmpty()) {
            return Optional.empty();
        }
        return Optional.of(new Context(instanceId, plugin.daemon().get()));
    }

    private static DaemonRequest request(String instanceId, String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId), command, body);
    }

    private record Context(String instanceId, DaemonClient client) {}
}
