package com.lkjmc.paper;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.ItemSpec;
import com.lkjmc.common.menu.MenuSpec;
import com.lkjmc.common.menu.StandardMenus;
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
        open(player, StandardMenus.root());
    }

    private void open(Player player, MenuSpec spec) {
        var locale = resolver.resolve(java.util.Optional.of(player.locale().toLanguageTag()));
        var inventory = Bukkit.createInventory(null, spec.size().slots(), catalog.render(locale, spec.title().key()));
        for (var slot : spec.slots()) {
            inventory.setItem(slot.slot(), item(locale, slot.item()));
        }
        player.openInventory(inventory);
    }

    private ItemStack item(String locale, ItemSpec spec) {
        var material = Material.matchMaterial(spec.material());
        var item = new ItemStack(material == null ? Material.STONE : material);
        var meta = item.getItemMeta();
        meta.setDisplayName(catalog.render(locale, spec.nameKey()));
        item.setItemMeta(meta);
        return item;
    }
}
