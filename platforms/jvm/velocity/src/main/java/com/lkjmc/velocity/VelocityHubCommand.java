package com.lkjmc.velocity;

import com.velocitypowered.api.command.SimpleCommand;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityHubCommand implements SimpleCommand {
    private final ProxyServer proxy;

    public VelocityHubCommand(ProxyServer proxy) {
        this.proxy = proxy;
    }

    @Override
    public void execute(Invocation invocation) {
        if (!(invocation.source() instanceof Player player)) {
            invocation.source().sendMessage(Component.text("players only", NamedTextColor.RED));
            return;
        }
        proxy.getServer("hub").ifPresentOrElse(
            server -> player.createConnectionRequest(server).fireAndForget(),
            () -> player.sendMessage(Component.text("hub unavailable", NamedTextColor.RED))
        );
    }
}
