package com.lkjmc.paper;

import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerTeleportEvent;
import org.bukkit.event.player.PlayerPortalEvent;

public final class PortalAccessListener implements Listener {
    private final RandomTeleportService randomTeleport;

    public PortalAccessListener(RandomTeleportService randomTeleport) {
        this.randomTeleport = randomTeleport;
    }

    @EventHandler
    public void onPortal(PlayerPortalEvent event) {
        var cause = event.getCause();
        if (cause == PlayerTeleportEvent.TeleportCause.NETHER_PORTAL
            || cause == PlayerTeleportEvent.TeleportCause.END_PORTAL) {
            event.setCancelled(true);
            randomTeleport.portalBlocked(event.getPlayer());
        }
    }
}
