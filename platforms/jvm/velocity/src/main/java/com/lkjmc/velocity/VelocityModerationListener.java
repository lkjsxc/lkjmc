package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.velocitypowered.api.event.EventTask;
import com.velocitypowered.api.event.ResultedEvent;
import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.connection.LoginEvent;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
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
            var raw = response.body().get("raw");
            if (response.ok() && raw != null && raw.toString().contains("\"banned\":true")) {
                event.setResult(ResultedEvent.ComponentResult.denied(Component.text("Banned: " + extract(raw.toString(), "reason"))));
            }
        });
        return EventTask.resumeWhenComplete(future.exceptionally(error -> null));
    }

    private static String extract(String json, String key) {
        var needle = "\"" + key + "\":\"";
        var start = json.indexOf(needle);
        if (start < 0) {
            return "";
        }
        var valueStart = start + needle.length();
        var end = json.indexOf('"', valueStart);
        return end < 0 ? "" : json.substring(valueStart, end);
    }
}
