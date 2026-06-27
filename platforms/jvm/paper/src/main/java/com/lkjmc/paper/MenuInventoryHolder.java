package com.lkjmc.paper;

import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuState;
import org.bukkit.inventory.Inventory;
import org.bukkit.inventory.InventoryHolder;

public final class MenuInventoryHolder implements InventoryHolder {
    private final MenuId menuId;
    private final MenuState state;
    private Inventory inventory;

    public MenuInventoryHolder(MenuId menuId, MenuState state) {
        this.menuId = menuId;
        this.state = state;
    }

    public void attach(Inventory inventory) {
        this.inventory = inventory;
    }

    public MenuId menuId() {
        return menuId;
    }

    public MenuState state() {
        return state;
    }

    @Override
    public Inventory getInventory() {
        return inventory;
    }
}
