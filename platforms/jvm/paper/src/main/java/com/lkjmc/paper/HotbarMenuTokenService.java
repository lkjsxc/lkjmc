package com.lkjmc.paper;

import org.bukkit.Material;
import org.bukkit.NamespacedKey;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

final class HotbarMenuTokenService {
    static final int SLOT = 8;
    private final NamespacedKey key;

    HotbarMenuTokenService(LkjmcPaperPlugin plugin) {
        this.key = new NamespacedKey(plugin, "local_docs_token");
    }

    boolean isToken(ItemStack item) {
        return item != null && item.hasItemMeta()
            && item.getItemMeta().getPersistentDataContainer().has(key, PersistentDataType.BYTE);
    }

    boolean isActiveToken(Player player, ItemStack item) {
        return player.getInventory().getHeldItemSlot() == SLOT && isToken(item);
    }

    ItemStack create() {
        var item = new ItemStack(Material.NETHER_STAR);
        var meta = item.getItemMeta();
        meta.setDisplayName("Documentation");
        meta.setLore(java.util.List.of("Open local lkjmc documentation"));
        meta.getPersistentDataContainer().set(key, PersistentDataType.BYTE, (byte) 1);
        item.setItemMeta(meta);
        return item;
    }
}
