package com.lkjmc.paper;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import java.util.Optional;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.block.Action;
import org.bukkit.event.entity.EntityPickupItemEvent;
import org.bukkit.event.inventory.InventoryClickEvent;
import org.bukkit.event.inventory.InventoryCloseEvent;
import org.bukkit.event.inventory.InventoryDragEvent;
import org.bukkit.event.player.PlayerDropItemEvent;
import org.bukkit.event.player.PlayerInteractAtEntityEvent;
import org.bukkit.event.player.PlayerInteractEntityEvent;
import org.bukkit.event.player.PlayerInteractEvent;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerRespawnEvent;
import org.bukkit.event.player.PlayerSwapHandItemsEvent;

public final class HotbarMenuListener implements Listener {
    private final MenuInventoryAdapter menus;
    private final MessageCatalog catalog;
    private final LocaleResolver resolver;
    private final HotbarMenuTokenService tokens;
    private final InventorySyncService sync;

    public HotbarMenuListener(MenuInventoryAdapter menus, MessageCatalog catalog, LocaleResolver resolver,
                              HotbarMenuTokenService tokens, InventorySyncService sync) {
        this.menus = menus;
        this.catalog = catalog;
        this.resolver = resolver;
        this.tokens = tokens;
        this.sync = sync;
    }

    @EventHandler public void onJoin(PlayerJoinEvent event) { sync.repairWithDelays(event.getPlayer()); }
    @EventHandler public void onRespawn(PlayerRespawnEvent event) { sync.repairWithDelays(event.getPlayer()); }
    @EventHandler public void onClose(InventoryCloseEvent event) { if (event.getPlayer() instanceof Player p) sync.repairWithDelays(p); }
    @EventHandler public void onPickup(EntityPickupItemEvent event) { if (event.getEntity() instanceof Player p) sync.repairWithDelays(p); }

    @EventHandler
    public void onInteract(PlayerInteractEvent event) {
        if (event.getAction() != Action.PHYSICAL && tokens.isActiveToken(event.getPlayer(), event.getItem())) {
            event.setCancelled(true);
            open(event.getPlayer());
        } else if (tokens.isToken(event.getItem())) {
            event.setCancelled(true);
            sync.repairNow(event.getPlayer());
        }
    }

    @EventHandler public void onEntity(PlayerInteractEntityEvent event) { if (activeHand(event.getPlayer())) { event.setCancelled(true); open(event.getPlayer()); } }
    @EventHandler public void onEntityAt(PlayerInteractAtEntityEvent event) { if (activeHand(event.getPlayer())) { event.setCancelled(true); open(event.getPlayer()); } }

    @EventHandler
    public void onDrop(PlayerDropItemEvent event) {
        if (!tokens.isToken(event.getItemDrop().getItemStack())) {
            return;
        }
        event.setCancelled(true);
        if (event.getPlayer().getInventory().getHeldItemSlot() == HotbarMenuTokenService.SLOT) {
            open(event.getPlayer());
        } else {
            sync.repairNow(event.getPlayer());
        }
    }

    @EventHandler
    public void onSwap(PlayerSwapHandItemsEvent event) {
        if (tokens.isToken(event.getMainHandItem()) || tokens.isToken(event.getOffHandItem())) {
            event.setCancelled(true);
            sync.repairNow(event.getPlayer());
        }
    }

    @EventHandler
    public void onClick(InventoryClickEvent event) {
        if (!(event.getWhoClicked() instanceof Player player)) {
            return;
        }
        if (event.getHotbarButton() == HotbarMenuTokenService.SLOT) {
            event.setCancelled(true);
            sync.repairNow(player);
            return;
        }
        if (isPlayerHotbarSlot(event, HotbarMenuTokenService.SLOT)) {
            event.setCancelled(true);
            if (tokens.isToken(event.getCurrentItem())) {
                open(player);
            } else {
                sync.repairNow(player);
            }
            return;
        }
        if (tokens.isToken(event.getCurrentItem()) || tokens.isToken(event.getCursor())) {
            event.setCancelled(true);
            sync.repairNow(player);
        }
    }

    @EventHandler
    public void onDrag(InventoryDragEvent event) {
        if (!(event.getWhoClicked() instanceof Player player)) {
            return;
        }
        if (event.getRawSlots().stream().anyMatch(raw -> isPlayerHotbarRaw(event, raw))) {
            event.setCancelled(true);
            sync.repairNow(player);
        }
    }

    private boolean isPlayerHotbarSlot(InventoryClickEvent event, int slot) {
        return event.getClickedInventory() != null
            && event.getClickedInventory().equals(event.getWhoClicked().getInventory())
            && event.getSlot() == slot;
    }

    private boolean isPlayerHotbarRaw(InventoryDragEvent event, int raw) {
        return raw >= event.getView().getTopInventory().getSize()
            && event.getView().convertSlot(raw) == HotbarMenuTokenService.SLOT;
    }

    private boolean activeHand(Player player) {
        return tokens.isActiveToken(player, player.getInventory().getItemInMainHand());
    }

    private void open(Player player) {
        try {
            menus.openRoot(player);
        } catch (RuntimeException error) {
            player.sendMessage(catalog.render(locale(player), "hotbar.menu.open-failed"));
        } finally {
            sync.repairNow(player);
        }
    }

    private String locale(Player player) { return resolver.resolve(Optional.of(player.locale().toLanguageTag())); }
}
