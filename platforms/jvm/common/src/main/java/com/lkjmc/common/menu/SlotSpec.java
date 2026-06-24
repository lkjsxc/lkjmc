package com.lkjmc.common.menu;

public record SlotSpec(int slot, ItemSpec item, MenuAction action) {
    public SlotSpec {
        if (slot < 0 || slot >= 54) {
            throw new IllegalArgumentException("slot out of range");
        }
        if (item == null) {
            throw new IllegalArgumentException("slot item is required");
        }
        if (action == null) {
            action = MenuAction.none();
        }
    }
}
