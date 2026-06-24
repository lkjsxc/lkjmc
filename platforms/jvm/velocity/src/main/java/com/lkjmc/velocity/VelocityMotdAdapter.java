package com.lkjmc.velocity;

import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.proxy.ProxyPingEvent;
import com.velocitypowered.api.proxy.server.ServerPing;
import net.kyori.adventure.text.Component;

public final class VelocityMotdAdapter {
    @Subscribe
    public void onProxyPing(ProxyPingEvent event) {
        event.setPing(event.getPing().asBuilder().description(render("lkjmc network")).build());
    }

    public Component render(String motd) {
        return Component.text(motd == null || motd.isBlank() ? "lkjmc" : motd);
    }

    public ServerPing withDescription(ServerPing ping, String motd) {
        return ping.asBuilder().description(render(motd)).build();
    }
}
