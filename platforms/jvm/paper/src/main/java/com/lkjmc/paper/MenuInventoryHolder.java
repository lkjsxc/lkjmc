package com.lkjmc.paper;

import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuSpec;
import com.lkjmc.common.menu.MenuState;
import org.bukkit.inventory.Inventory;
import org.bukkit.inventory.InventoryHolder;

public final class MenuInventoryHolder implements InventoryHolder {
    private final MenuSpec spec;
    private final MenuState state;
    private Inventory inventory;

    public MenuInventoryHolder(MenuSpec spec, MenuState state) {
        this.spec = spec;
        this.state = state;
    }

    public void attach(Inventory inventory) {
        this.inventory = inventory;
    }

    public MenuId menuId() {
        return spec.id();
    }

    public MenuSpec spec() {
        return spec;
    }

    public MenuState state() {
        return state;
    }

    @Override
    public Inventory getInventory() {
        return inventory;
    }
}
