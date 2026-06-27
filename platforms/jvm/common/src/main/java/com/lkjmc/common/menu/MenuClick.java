package com.lkjmc.common.menu;

public record MenuClick(int slot, MenuMetadata metadata, String actionKey, boolean topInventory) {
    public MenuClick(int slot) {
        this(slot, null, null, true);
    }

    public MenuClick(int slot, String actionKey, boolean topInventory) {
        this(slot, null, actionKey, topInventory);
    }

    public String effectiveActionKey() {
        return metadata == null ? actionKey : metadata.actionKey();
    }
}
