package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.velocitypowered.api.event.EventTask;
import com.velocitypowered.api.event.ResultedEvent;
import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.connection.LoginEvent;
import java.util.Map;
import java.util.UUID;
import net.kyori.adventure.text.Component;

public final class VelocityModerationListener {
    private final DaemonClient daemon;

    public VelocityModerationListener(DaemonClient daemon) {
        this.daemon = daemon;
    }

    @Subscribe
    public EventTask onLogin(LoginEvent event) {
        var player = event.getPlayer();
        var request = new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("velocity-plugin", "velocity"),
            "player.moderation.status",
            Map.of("playerUuid", player.getUniqueId().toString(), "playerName", player.getUsername())
        );
        var future = daemon.send(request).thenAccept(response -> {
            if (response.ok() && DaemonJson.bool(response.body(), "banned")) {
                var reason = DaemonJson.string(response.body(), "reason").orElse("");
                event.setResult(ResultedEvent.ComponentResult.denied(Component.text("Banned: " + reason)));
            }
        });
        return EventTask.resumeWhenComplete(future.exceptionally(error -> null));
    }
}
