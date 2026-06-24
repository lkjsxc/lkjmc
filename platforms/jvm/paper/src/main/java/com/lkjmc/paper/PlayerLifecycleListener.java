package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
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
        var context = context();
        if (context.isEmpty()) {
            return;
        }
        var playerId = event.getPlayer().getUniqueId().toString();
        var request = request(context.get().instanceId(), "player.load", Map.of(
            "playerUuid", playerId,
            "scope", "profile"
        ));
        context.get().client().send(request).thenAccept(response -> Optional.ofNullable(response.body().get("raw"))
            .map(Object::toString)
            .flatMap(PlayerLifecycleListener::extractPayload)
            .ifPresent(payload -> apply(event.getPlayer(), payload)));
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
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
    }

    private void apply(Player player, String payloadBase64) {
        plugin.scheduler().runPlayer(player, () -> profiles.apply(player, payloadBase64));
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

    private static Optional<String> extractPayload(String raw) {
        return extract(raw, "payloadBase64");
    }

    private static Optional<String> extract(String json, String key) {
        var needle = "\"" + key + "\":\"";
        var start = json.indexOf(needle);
        if (start < 0) {
            return Optional.empty();
        }
        var valueStart = start + needle.length();
        var end = json.indexOf('"', valueStart);
        return end < 0 ? Optional.empty() : Optional.of(json.substring(valueStart, end));
    }

    private record Context(String instanceId, DaemonClient client) {}
}
