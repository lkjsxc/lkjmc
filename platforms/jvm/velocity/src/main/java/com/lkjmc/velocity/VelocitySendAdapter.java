package com.lkjmc.velocity;

import com.velocitypowered.api.command.SimpleCommand;
import com.velocitypowered.api.proxy.ProxyServer;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocitySendAdapter {
    private final ProxyServer proxy;
    private final ProfileSaveBridge transfers;
    private final VelocityTransferCoordinator coordinator = new VelocityTransferCoordinator();

    public VelocitySendAdapter(ProxyServer proxy, ProfileSaveBridge transfers) {
        this.proxy = proxy;
        this.transfers = transfers;
    }

    public void send(SimpleCommand.Invocation invocation, String playerName, String serverName) {
        var player = proxy.getPlayer(playerName);
        var target = proxy.getServer(serverName);
        if (player.isEmpty() || target.isEmpty()) {
            invocation.source().sendMessage(Component.text("player or server unavailable", NamedTextColor.RED));
            return;
        }
        var source = player.get().getCurrentServer()
            .map(server -> server.getServerInfo().getName())
            .orElse("");
        if (!coordinator.canTransfer(source, serverName)) {
            invocation.source().sendMessage(Component.text("transfer denied", NamedTextColor.RED));
            return;
        }
        transfers.save(player.get()).thenAccept(saved -> {
            if (!saved) {
                invocation.source().sendMessage(Component.text("source save timed out", NamedTextColor.RED));
                return;
            }
            player.get().createConnectionRequest(target.get()).fireAndForget();
            invocation.source().sendMessage(Component.text("transfer requested", NamedTextColor.GREEN));
        });
    }
}
