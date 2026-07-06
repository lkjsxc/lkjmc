package com.lkjmc.paper.ui;

import com.lkjmc.common.ui.kernel.MenuRoute;
import org.bukkit.inventory.Inventory;
import org.bukkit.inventory.InventoryHolder;

public final class UiInventoryHolder implements InventoryHolder {
    private final String sessionId;
    private final MenuRoute route;
    private final int size;
    private Inventory inventory;

    UiInventoryHolder(String sessionId, MenuRoute route, int size) {
        this.sessionId = sessionId == null ? "" : sessionId;
        this.route = route == null ? MenuRoute.root() : route;
        this.size = size;
    }

    void attach(Inventory inventory) {
        this.inventory = inventory;
    }

    public String sessionId() {
        return sessionId;
    }

    public MenuRoute route() {
        return route;
    }

    public int size() {
        return size;
    }

    @Override
    public Inventory getInventory() {
        return inventory;
    }
}
