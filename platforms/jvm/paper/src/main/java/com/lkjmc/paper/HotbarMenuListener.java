package com.lkjmc.paper;

import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.entity.EntityPickupItemEvent;
import org.bukkit.event.inventory.InventoryClickEvent;
import org.bukkit.event.inventory.InventoryDragEvent;
import org.bukkit.event.player.PlayerDropItemEvent;
import org.bukkit.event.player.PlayerInteractEvent;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerRespawnEvent;

public final class HotbarMenuListener implements Listener {
    private final LocalDocsMenu docs;
    private final HotbarMenuTokenService tokens;
    private final InventorySyncService sync;

    HotbarMenuListener(LocalDocsMenu docs, HotbarMenuTokenService tokens, InventorySyncService sync) {
        this.docs = docs;
        this.tokens = tokens;
        this.sync = sync;
    }

    @EventHandler public void onJoin(PlayerJoinEvent event) { sync.repair(event.getPlayer()); }
    @EventHandler public void onRespawn(PlayerRespawnEvent event) { sync.repair(event.getPlayer()); }
    @EventHandler public void onPickup(EntityPickupItemEvent event) {
        if (event.getEntity() instanceof org.bukkit.entity.Player player) sync.repair(player);
    }

    @EventHandler
    public void onInteract(PlayerInteractEvent event) {
        if (tokens.isActiveToken(event.getPlayer(), event.getItem())) {
            event.setCancelled(true);
            docs.openRoot(event.getPlayer());
        }
    }

    @EventHandler
    public void onDrop(PlayerDropItemEvent event) {
        if (tokens.isToken(event.getItemDrop().getItemStack())) {
            event.setCancelled(true);
            docs.openRoot(event.getPlayer());
        }
    }

    @EventHandler
    public void onClick(InventoryClickEvent event) {
        if (!(event.getWhoClicked() instanceof org.bukkit.entity.Player player)) return;
        if (event.getHotbarButton() == HotbarMenuTokenService.SLOT
            || tokens.isToken(event.getCurrentItem()) || tokens.isToken(event.getCursor())) {
            event.setCancelled(true);
            sync.repair(player);
        }
    }

    @EventHandler
    public void onDrag(InventoryDragEvent event) {
        if (event.getRawSlots().stream().anyMatch(slot -> slot == HotbarMenuTokenService.SLOT)) {
            event.setCancelled(true);
        }
    }
}
