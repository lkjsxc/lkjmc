package com.lkjmc.common.menu;

public enum MenuFailure {
    UNKNOWN_METADATA("menu.error.unknown-action"),
    STALE_SESSION("menu.error.stale-session"),
    STALE_EPOCH("menu.error.stale-epoch"),
    ROUTE_MISMATCH("menu.error.route-mismatch"),
    UNHANDLED_ACTION("menu.error.unhandled-action");

    private final String messageKey;

    MenuFailure(String messageKey) {
        this.messageKey = messageKey;
    }

    public String messageKey() {
        return messageKey;
    }
}
