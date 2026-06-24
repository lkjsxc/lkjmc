package com.lkjmc.common.menu;

public record MenuState(MenuId current, int page) {
    public MenuState {
        if (page < 0) {
            throw new IllegalArgumentException("page must be non-negative");
        }
    }
}
