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
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityTemporarySendAdapter {
    private final ProxyServer proxy;
    private final Optional<DaemonClient> daemon;
    private final VelocitySendAdapter send;

    public VelocityTemporarySendAdapter(
        ProxyServer proxy,
        Optional<DaemonClient> daemon,
        VelocitySendAdapter send
    ) {
        this.proxy = proxy;
        this.daemon = daemon;
        this.send = send;
    }

    public void send(CommandSource sender, String playerName, String instanceId) {
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
            "temporaryInstanceId", instanceId
        );
        var request = new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("velocity-plugin", "velocity"),
            "temporary.transfer.intent",
            body
        );
        daemon.get().send(request).thenAccept(response -> {
            if (!response.ok()) {
                sender.sendMessage(Component.text(
                    response.error().map(Object::toString).orElse("temporary transfer denied"),
                    NamedTextColor.RED
                ));
                return;
            }
            var target = DaemonJson.string(response.body(), "targetServer").orElse(instanceId);
            send.send(sender, player.get().getUsername(), target);
        });
    }
}
