package com.lkjmc.paper;

import org.bukkit.inventory.Inventory;
import org.bukkit.inventory.InventoryHolder;

final class DocsMenuHolder implements InventoryHolder {
    private final String route;

    DocsMenuHolder(String route) {
        this.route = route;
    }

    String route() {
        return route;
    }

    @Override
    public Inventory getInventory() {
        return null;
    }
}
