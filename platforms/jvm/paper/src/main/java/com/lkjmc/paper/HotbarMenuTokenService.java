package com.lkjmc.paper;

import com.lkjmc.common.i18n.MessageCatalog;
import java.util.HashSet;
import java.util.Set;
import java.util.UUID;
import org.bukkit.Material;
import org.bukkit.NamespacedKey;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

final class HotbarMenuTokenService {
    static final int SLOT = 8;
    static final Material TOKEN_MATERIAL = Material.NETHER_STAR;
    static final String MARKER_KEY = "menu_item";
    private final LkjmcPaperPlugin plugin;
    private final MessageCatalog catalog;
    private final NamespacedKey key;
    private final Set<UUID> disabled = new HashSet<>();

    HotbarMenuTokenService(LkjmcPaperPlugin plugin, MessageCatalog catalog) {
        this.plugin = plugin;
        this.catalog = catalog;
        this.key = new NamespacedKey(plugin, MARKER_KEY);
    }

    boolean isToken(ItemStack item) {
        return item != null && item.hasItemMeta()
            && item.getItemMeta().getPersistentDataContainer().has(key, PersistentDataType.BYTE);
    }

    boolean isActiveToken(Player player, ItemStack item) {
        return enabled(player) && player.getInventory().getHeldItemSlot() == SLOT && isToken(item);
    }

    boolean enabled(Player player) {
        return !disabled.contains(player.getUniqueId());
    }

    void setEnabled(Player player, boolean enabled) {
        if (enabled) {
            disabled.remove(player.getUniqueId());
        } else {
            disabled.add(player.getUniqueId());
        }
    }

    ItemStack create(Player player) {
        var item = new ItemStack(TOKEN_MATERIAL);
        var meta = item.getItemMeta();
        meta.setDisplayName(catalog.render(locale(player), "hotbar.menu.name"));
        meta.setLore(java.util.List.of(catalog.render(locale(player), "hotbar.menu.lore")));
        meta.getPersistentDataContainer().set(key, PersistentDataType.BYTE, (byte) 1);
        item.setItemMeta(meta);
        return item;
    }

    private String locale(Player player) {
        return plugin.localeService().locale(player);
    }
}
