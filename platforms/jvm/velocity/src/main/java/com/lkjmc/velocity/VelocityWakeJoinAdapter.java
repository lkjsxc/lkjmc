package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.velocitypowered.api.command.CommandSource;
import com.velocitypowered.api.proxy.ProxyServer;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityWakeJoinAdapter {
    private final ProxyServer proxy;
    private final Optional<DaemonClient> daemon;
    private final Optional<VelocityServerRegistry> registry;
    private final VelocitySendAdapter send;

    public VelocityWakeJoinAdapter(
        ProxyServer proxy,
        Optional<DaemonClient> daemon,
        Optional<VelocityServerRegistry> registry,
        VelocitySendAdapter send
    ) {
        this.proxy = proxy;
        this.daemon = daemon;
        this.registry = registry;
        this.send = send;
    }

    public void send(CommandSource sender, String playerName, String target) {
        if (daemon.isEmpty()) {
            sender.sendMessage(Component.text("daemon HTTP is not configured", NamedTextColor.RED));
            return;
        }
        var player = proxy.getPlayer(playerName);
        if (player.isEmpty()) {
            sender.sendMessage(Component.text("player unavailable", NamedTextColor.RED));
            return;
        }
        var body = Map.<String, Object>of(
            "playerUuid", player.get().getUniqueId().toString(),
            "playerName", player.get().getUsername(),
            "targetInstanceId", target
        );
        daemon.get().send(request(body)).thenAccept(response -> {
            if (!response.ok()) {
                sender.sendMessage(Component.text(
                    response.error().map(Object::toString).orElse("wake failed"), NamedTextColor.RED));
                return;
            }
            var queueId = DaemonJson.string(response.body(), "queueId").orElse("");
            var targetServer = DaemonJson.string(response.body(), "targetServer").orElse(target);
            refresh().thenRun(() -> consumeAndSend(sender, player.get().getUsername(), queueId, targetServer));
        });
    }

    private void consumeAndSend(CommandSource sender, String playerName, String queueId, String target) {
        if (queueId.isBlank() || proxy.getServer(target).isEmpty()) {
            sender.sendMessage(Component.text("wake target unavailable", NamedTextColor.RED));
            return;
        }
        var body = Map.<String, Object>of("queueId", queueId, "targetServer", target);
        daemon.get().send(request("instance.wake.consume", body)).thenAccept(response -> {
            if (response.ok()) {
                send.send(sender, playerName, target);
            } else {
                sender.sendMessage(Component.text("wake request was already consumed", NamedTextColor.RED));
            }
        });
    }

    private CompletableFuture<Void> refresh() {
        return registry.map(VelocityServerRegistry::refresh).orElseGet(() -> CompletableFuture.completedFuture(null));
    }

    private static DaemonRequest request(Map<String, Object> body) {
        return request("instance.wake.request", body);
    }

    private static DaemonRequest request(String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("velocity-plugin", "velocity"), command, body);
    }
}
