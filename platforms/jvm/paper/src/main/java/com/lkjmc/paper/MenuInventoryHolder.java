package com.lkjmc.paper;

import com.lkjmc.common.menu.MenuId;
import java.util.UUID;
import org.bukkit.inventory.Inventory;
import org.bukkit.inventory.InventoryHolder;

public final class MenuInventoryHolder implements InventoryHolder {
    private final MenuId menuId;
    private final UUID sessionId;
    private Inventory inventory;

    public MenuInventoryHolder(MenuId menuId, UUID sessionId) {
        this.menuId = menuId;
        this.sessionId = sessionId;
    }

    public void attach(Inventory inventory) {
        this.inventory = inventory;
    }

    public MenuId menuId() {
        return menuId;
    }

    public UUID sessionId() {
        return sessionId;
    }

    @Override
    public Inventory getInventory() {
        return inventory;
    }
}
