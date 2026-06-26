package com.lkjmc.common.menu;

public record MenuClick(int slot, String actionKey, boolean topInventory) {
    public MenuClick(int slot) {
        this(slot, null, true);
    }
}
