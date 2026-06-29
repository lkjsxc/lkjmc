package com.lkjmc.velocity;

import com.velocitypowered.api.command.CommandSource;
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

    public void send(CommandSource sender, String playerName, String serverName) {
        var player = proxy.getPlayer(playerName);
        var target = proxy.getServer(serverName);
        if (player.isEmpty() || target.isEmpty()) {
            sender.sendMessage(Component.text("player or server unavailable", NamedTextColor.RED));
            return;
        }
        var source = player.get().getCurrentServer()
            .map(server -> server.getServerInfo().getName())
            .orElse("");
        if (!coordinator.canTransfer(source, serverName)) {
            sender.sendMessage(Component.text("transfer denied", NamedTextColor.RED));
            return;
        }
        transfers.save(player.get()).thenAccept(saved -> {
            if (!saved) {
                sender.sendMessage(Component.text("source save timed out", NamedTextColor.RED));
                return;
            }
            player.get().createConnectionRequest(target.get()).fireAndForget();
            sender.sendMessage(Component.text("transfer requested", NamedTextColor.GREEN));
        });
    }
}
