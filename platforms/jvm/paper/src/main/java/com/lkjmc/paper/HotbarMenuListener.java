package com.lkjmc.paper;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import java.util.Optional;
import org.bukkit.Material;
import org.bukkit.NamespacedKey;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.block.Action;
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
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

public final class HotbarMenuListener implements Listener {
    private static final int SLOT = 8;
    private final LkjmcPaperPlugin plugin;
    private final MenuInventoryAdapter menus;
    private final MessageCatalog catalog;
    private final LocaleResolver resolver;
    private final NamespacedKey key;

    public HotbarMenuListener(LkjmcPaperPlugin plugin, MenuInventoryAdapter menus, MessageCatalog catalog, LocaleResolver resolver) {
        this.plugin = plugin;
        this.menus = menus;
        this.catalog = catalog;
        this.resolver = resolver;
        this.key = new NamespacedKey(plugin, "menu_item");
    }

    @EventHandler
    public void onJoin(PlayerJoinEvent event) { syncLater(event.getPlayer()); }

    @EventHandler
    public void onRespawn(PlayerRespawnEvent event) { syncLater(event.getPlayer()); }

    @EventHandler
    public void onClose(InventoryCloseEvent event) {
        if (event.getPlayer() instanceof Player player) { syncLater(player); }
    }

    @EventHandler
    public void onInteract(PlayerInteractEvent event) {
        if (event.getAction() != Action.PHYSICAL && isActiveToken(event.getPlayer(), event.getItem())) {
            event.setCancelled(true);
            open(event.getPlayer());
        } else if (isMenuItem(event.getItem())) {
            event.setCancelled(true);
            syncLater(event.getPlayer());
        }
    }

    @EventHandler
    public void onEntity(PlayerInteractEntityEvent event) {
        if (isActiveEntityToken(event.getPlayer())) { event.setCancelled(true); open(event.getPlayer()); }
    }

    @EventHandler
    public void onEntityAt(PlayerInteractAtEntityEvent event) {
        if (isActiveEntityToken(event.getPlayer())) { event.setCancelled(true); open(event.getPlayer()); }
    }

    @EventHandler
    public void onDrop(PlayerDropItemEvent event) {
        if (isMenuItem(event.getItemDrop().getItemStack())) {
            event.setCancelled(true);
            if (event.getPlayer().getInventory().getHeldItemSlot() == SLOT) { open(event.getPlayer()); }
            else { syncLater(event.getPlayer()); }
        }
    }

    @EventHandler
    public void onSwap(PlayerSwapHandItemsEvent event) {
        if (isMenuItem(event.getMainHandItem()) || isMenuItem(event.getOffHandItem())) {
            event.setCancelled(true);
            syncLater(event.getPlayer());
        }
    }

    @EventHandler
    public void onClick(InventoryClickEvent event) {
        if (!(event.getWhoClicked() instanceof Player player)) { return; }
        if (event.getHotbarButton() == SLOT) { event.setCancelled(true); syncLater(player); return; }
        if (isPlayerHotbarSlot(event, SLOT)) {
            event.setCancelled(true);
            if (isMenuItem(event.getCurrentItem())) { open(player); } else { syncLater(player); }
            return;
        }
        if (isMenuItem(event.getCurrentItem()) || isMenuItem(event.getCursor())) {
            event.setCancelled(true);
            syncLater(player);
        }
    }

    @EventHandler
    public void onDrag(InventoryDragEvent event) {
        if (!(event.getWhoClicked() instanceof Player player)) { return; }
        if (event.getRawSlots().stream().anyMatch(raw -> isPlayerHotbarRaw(event, raw, SLOT))) {
            event.setCancelled(true);
            syncLater(player);
        }
    }

    private boolean isPlayerHotbarSlot(InventoryClickEvent event, int slot) {
        return event.getClickedInventory() != null
            && event.getClickedInventory().equals(event.getWhoClicked().getInventory())
            && event.getSlot() == slot;
    }

    private boolean isPlayerHotbarRaw(InventoryDragEvent event, int raw, int slot) {
        return raw >= event.getView().getTopInventory().getSize()
            && event.getView().convertSlot(raw) == slot;
    }

    private void open(Player player) {
        try { menus.openRoot(player); } catch (RuntimeException error) {
            player.sendMessage(catalog.render(locale(player), "hotbar.menu.open-failed"));
        } finally { syncLater(player); }
    }

    private void syncLater(Player player) { plugin.scheduler().runPlayer(player, () -> sync(player)); }

    private void sync(Player player) {
        for (int index = 0; index < player.getInventory().getSize(); index++) {
            if (index != SLOT && isMenuItem(player.getInventory().getItem(index))) {
                player.getInventory().setItem(index, null);
            }
        }
        player.getInventory().setItem(SLOT, menuItem(player));
    }

    private boolean isActiveToken(Player player, ItemStack item) {
        return player.getInventory().getHeldItemSlot() == SLOT && isMenuItem(item);
    }

    private boolean isActiveEntityToken(Player player) {
        return isActiveToken(player, player.getInventory().getItemInMainHand());
    }

    private ItemStack menuItem(Player player) {
        var item = new ItemStack(Material.COMPASS);
        var meta = item.getItemMeta();
        meta.setDisplayName(catalog.render(locale(player), "hotbar.menu.name"));
        meta.setLore(java.util.List.of(catalog.render(locale(player), "hotbar.menu.lore")));
        meta.getPersistentDataContainer().set(key, PersistentDataType.BYTE, (byte) 1);
        item.setItemMeta(meta);
        return item;
    }

    private boolean isMenuItem(ItemStack item) {
        return item != null && item.hasItemMeta()
            && item.getItemMeta().getPersistentDataContainer().has(key, PersistentDataType.BYTE);
    }

    private String locale(Player player) { return resolver.resolve(Optional.of(player.locale().toLanguageTag())); }
}
