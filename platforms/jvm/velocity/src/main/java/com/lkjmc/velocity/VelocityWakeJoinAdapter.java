package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.velocitypowered.api.command.SimpleCommand;
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

    public void send(SimpleCommand.Invocation invocation, String playerName, String target) {
        if (daemon.isEmpty()) {
            invocation.source().sendMessage(Component.text("daemon HTTP is not configured", NamedTextColor.RED));
            return;
        }
        var player = proxy.getPlayer(playerName);
        if (player.isEmpty()) {
            invocation.source().sendMessage(Component.text("player unavailable", NamedTextColor.RED));
            return;
        }
        var body = Map.<String, Object>of(
            "playerUuid", player.get().getUniqueId().toString(),
            "playerName", player.get().getUsername(),
            "targetInstanceId", target
        );
        daemon.get().send(request(body)).thenAccept(response -> {
            if (!response.ok()) {
                invocation.source().sendMessage(Component.text(
                    response.error().map(Object::toString).orElse("wake failed"), NamedTextColor.RED));
                return;
            }
            var targetServer = DaemonJson.string(response.body(), "targetServer").orElse(target);
            refresh().thenRun(() -> send.send(invocation, player.get().getUsername(), targetServer));
        });
    }

    private CompletableFuture<Void> refresh() {
        return registry.map(VelocityServerRegistry::refresh).orElseGet(() -> CompletableFuture.completedFuture(null));
    }

    private static DaemonRequest request(Map<String, Object> body) {
        return new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("velocity-plugin", "velocity"), "instance.wake.request", body
        );
    }
}
