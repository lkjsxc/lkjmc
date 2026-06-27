package com.lkjmc.common.menu;

public record MenuSlot(int value) {
    public MenuSlot {
        if (value < 0 || value >= 54) {
            throw new IllegalArgumentException("menu slot out of range");
        }
    }
}
