package com.lkjmc.velocity;

import com.velocitypowered.api.command.SimpleCommand;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityHubCommand implements SimpleCommand {
    private final ProxyServer proxy;
    private final ProfileSaveBridge transfers;
    private final VelocityTransferCoordinator coordinator = new VelocityTransferCoordinator();

    public VelocityHubCommand(ProxyServer proxy, ProfileSaveBridge transfers) {
        this.proxy = proxy;
        this.transfers = transfers;
    }

    @Override
    public void execute(Invocation invocation) {
        if (!(invocation.source() instanceof Player player)) {
            invocation.source().sendMessage(Component.text("players only", NamedTextColor.RED));
            return;
        }
        proxy.getServer("hub").ifPresentOrElse(server -> transfers.save(player).thenAccept(saved -> {
            if (!saved) {
                player.sendMessage(Component.text("source save timed out", NamedTextColor.RED));
                return;
            }
            coordinator.connect(player, server).thenAccept(connected -> player.sendMessage(Component.text(
                connected ? "hub transfer completed" : "hub transfer failed",
                connected ? NamedTextColor.GREEN : NamedTextColor.RED)));
        }), () -> player.sendMessage(Component.text("hub unavailable", NamedTextColor.RED)));
    }
}
