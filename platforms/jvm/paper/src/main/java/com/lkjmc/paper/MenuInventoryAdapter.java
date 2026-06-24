package com.lkjmc.paper;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import org.bukkit.Bukkit;
import org.bukkit.Material;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;

public final class MenuInventoryAdapter {
    private final MessageCatalog catalog;
    private final LocaleResolver resolver;

    public MenuInventoryAdapter(MessageCatalog catalog, LocaleResolver resolver) {
        this.catalog = catalog;
        this.resolver = resolver;
    }

    public void openRoot(Player player) {
        var locale = resolver.resolve(java.util.Optional.of(player.locale().toLanguageTag()));
        var inventory = Bukkit.createInventory(null, 54, catalog.render(locale, "menu.root.title"));
        var item = new ItemStack(Material.COMPASS);
        var meta = item.getItemMeta();
        meta.setDisplayName(catalog.render(locale, "server.status.header"));
        item.setItemMeta(meta);
        inventory.setItem(4, item);
        player.openInventory(inventory);
    }
}
