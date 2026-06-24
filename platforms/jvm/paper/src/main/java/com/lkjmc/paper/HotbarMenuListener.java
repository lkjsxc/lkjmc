package com.lkjmc.paper;

import org.bukkit.Material;
import org.bukkit.NamespacedKey;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.block.Action;
import org.bukkit.event.inventory.InventoryClickEvent;
import org.bukkit.event.inventory.InventoryDragEvent;
import org.bukkit.event.player.PlayerDropItemEvent;
import org.bukkit.event.player.PlayerInteractEvent;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerSwapHandItemsEvent;
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

public final class HotbarMenuListener implements Listener {
    private static final int SLOT = 8;
    private final LkjmcPaperPlugin plugin;
    private final MenuInventoryAdapter menus;
    private final NamespacedKey key;

    public HotbarMenuListener(LkjmcPaperPlugin plugin, MenuInventoryAdapter menus) {
        this.plugin = plugin;
        this.menus = menus;
        this.key = new NamespacedKey(plugin, "menu_item");
    }

    @EventHandler
    public void onJoin(PlayerJoinEvent event) {
        plugin.scheduler().runPlayer(event.getPlayer(), () -> give(event.getPlayer()));
    }

    @EventHandler
    public void onInteract(PlayerInteractEvent event) {
        if (isMenuItem(event.getItem()) && event.getAction() != Action.PHYSICAL) {
            event.setCancelled(true);
            menus.openRoot(event.getPlayer());
        }
    }

    @EventHandler
    public void onDrop(PlayerDropItemEvent event) {
        if (isMenuItem(event.getItemDrop().getItemStack())) {
            event.setCancelled(true);
        }
    }

    @EventHandler
    public void onSwap(PlayerSwapHandItemsEvent event) {
        if (isMenuItem(event.getMainHandItem()) || isMenuItem(event.getOffHandItem())) {
            event.setCancelled(true);
        }
    }

    @EventHandler
    public void onClick(InventoryClickEvent event) {
        if (isMenuItem(event.getCurrentItem()) || event.getHotbarButton() == SLOT) {
            event.setCancelled(true);
        }
    }

    @EventHandler
    public void onDrag(InventoryDragEvent event) {
        if (event.getRawSlots().contains(SLOT)) {
            event.setCancelled(true);
        }
    }

    private void give(Player player) {
        player.getInventory().setItem(SLOT, menuItem());
    }

    private ItemStack menuItem() {
        var item = new ItemStack(Material.COMPASS);
        var meta = item.getItemMeta();
        meta.setDisplayName("Menu");
        meta.getPersistentDataContainer().set(key, PersistentDataType.BYTE, (byte) 1);
        item.setItemMeta(meta);
        return item;
    }

    private boolean isMenuItem(ItemStack item) {
        return item != null && item.hasItemMeta()
            && item.getItemMeta().getPersistentDataContainer().has(key, PersistentDataType.BYTE);
    }
}
