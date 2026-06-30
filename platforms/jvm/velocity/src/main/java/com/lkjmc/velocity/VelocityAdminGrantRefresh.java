package com.lkjmc.velocity;

import com.lkjmc.common.permission.PermissionSnapshotCache;
import com.lkjmc.common.permission.PrincipalIdentity;
import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.connection.DisconnectEvent;
import com.velocitypowered.api.event.connection.PostLoginEvent;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import java.time.Duration;

final class VelocityAdminGrantRefresh {
    private final ProxyServer proxy;
    private final PermissionSnapshotCache cache;

    VelocityAdminGrantRefresh(ProxyServer proxy, PermissionSnapshotCache cache) {
        this.proxy = proxy;
        this.cache = cache == null ? PermissionSnapshotCache.disabled() : cache;
    }

    void register(Object plugin) {
        proxy.getEventManager().register(plugin, this);
        proxy.getScheduler().buildTask(plugin, this::refreshOnline)
            .repeat(Duration.ofSeconds(30)).schedule();
    }

    @Subscribe
    public void onPostLogin(PostLoginEvent event) {
        refresh(event.getPlayer());
    }

    @Subscribe
    public void onDisconnect(DisconnectEvent event) {
        cache.evict(identity(event.getPlayer()));
    }

    private void refreshOnline() {
        proxy.getAllPlayers().forEach(this::refresh);
    }

    private void refresh(Player player) {
        cache.refresh(identity(player)).exceptionally(error -> null);
    }

    private static PrincipalIdentity identity(Player player) {
        return new PrincipalIdentity("minecraft-player", player.getUniqueId().toString(), player.getUsername());
    }
}
