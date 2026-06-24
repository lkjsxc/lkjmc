package com.lkjmc.velocity;

import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.connection.PostLoginEvent;
import com.velocitypowered.api.proxy.ProxyServer;
import net.kyori.adventure.text.Component;

public final class VelocityTabListAdapter {
    private final ProxyServer proxy;

    public VelocityTabListAdapter(ProxyServer proxy) {
        this.proxy = proxy;
    }

    @Subscribe
    public void onPostLogin(PostLoginEvent event) {
        event.getPlayer().sendPlayerListHeaderAndFooter(header(proxy.getPlayerCount()), footer());
    }

    public Component header(int onlinePlayers) {
        return Component.text("lkjmc players: " + onlinePlayers);
    }

    public Component footer() {
        return Component.text("managed by lkjmc");
    }
}
