package com.lkjmc.common.menu;

public record MenuActionPayload(String value) {
    public static final MenuActionPayload EMPTY = new MenuActionPayload("");

    public MenuActionPayload {
        value = value == null ? "" : value;
    }
}
