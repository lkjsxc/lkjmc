package com.lkjmc.paper;

import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.MenuSpec;
import com.lkjmc.common.menu.MenuState;
import org.bukkit.Bukkit;
import org.bukkit.inventory.Inventory;

final class MenuInventoryRenderer {
    private final MessageCatalog catalog;
    private final MenuItemFactory items;

    MenuInventoryRenderer(MessageCatalog catalog, MenuItemFactory items) {
        this.catalog = catalog;
        this.items = items;
    }

    Inventory render(String locale, MenuSpec spec, MenuState state) {
        var holder = new MenuInventoryHolder(spec.id(), state);
        var inventory = Bukkit.createInventory(holder, spec.size().slots(), catalog.render(locale, spec.title().key()));
        holder.attach(inventory);
        for (var slot : spec.slots()) {
            inventory.setItem(slot.slot(), items.item(locale, state, slot.slot(), slot.item(), slot.action()));
        }
        return inventory;
    }
}
