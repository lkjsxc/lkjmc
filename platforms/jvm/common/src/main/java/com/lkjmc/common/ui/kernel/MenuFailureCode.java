package com.lkjmc.common.ui.kernel;

public enum MenuFailureCode {
    UNKNOWN_METADATA("menu.error.unknown-action"),
    STALE_SESSION("menu.error.stale-session"),
    STALE_EPOCH("menu.error.stale-epoch"),
    ROUTE_MISMATCH("menu.error.route-mismatch");

    private final String messageKey;

    MenuFailureCode(String messageKey) {
        this.messageKey = messageKey;
    }

    public String messageKey() {
        return messageKey;
    }
}
