package com.lkjmc.paper;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import java.util.HashSet;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import org.bukkit.Material;
import org.bukkit.NamespacedKey;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

final class HotbarMenuTokenService {
    static final int SLOT = 8;
    private final MessageCatalog catalog;
    private final LocaleResolver resolver;
    private final NamespacedKey key;
    private final Set<UUID> disabled = new HashSet<>();

    HotbarMenuTokenService(LkjmcPaperPlugin plugin, MessageCatalog catalog, LocaleResolver resolver) {
        this.catalog = catalog;
        this.resolver = resolver;
        this.key = new NamespacedKey(plugin, "menu_item");
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
        var item = new ItemStack(Material.COMPASS);
        var meta = item.getItemMeta();
        meta.setDisplayName(catalog.render(locale(player), "hotbar.menu.name"));
        meta.setLore(java.util.List.of(catalog.render(locale(player), "hotbar.menu.lore")));
        meta.getPersistentDataContainer().set(key, PersistentDataType.BYTE, (byte) 1);
        item.setItemMeta(meta);
        return item;
    }

    private String locale(Player player) {
        return resolver.resolve(Optional.of(player.locale().toLanguageTag()));
    }
}
