package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import java.util.Map;
import java.util.UUID;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerQuitEvent;

public final class PlayerLifecycleListener implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final PlayerProfileAdapter profiles = new PlayerProfileAdapter();

    public PlayerLifecycleListener(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        var instanceId = System.getenv("LKJMC_INSTANCE_ID");
        if (instanceId == null || instanceId.isBlank() || plugin.daemon().isEmpty()) {
            return;
        }
        var snapshot = profiles.capture(event.getPlayer());
        var request = new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId),
            "player.snapshot",
            Map.of(
                "playerUuid", event.getPlayer().getUniqueId().toString(),
                "name", event.getPlayer().getName(),
                "sourceInstance", instanceId,
                "scope", "profile",
                "payloadBase64", snapshot.payloadBase64(),
                "sha256", snapshot.sha256()
            )
        );
        plugin.daemon().get().send(request);
    }
}
