package com.lkjmc.common.menu;

public record MenuMetadata(
    MenuId menuId,
    MenuRoute route,
    int slot,
    String actionKey,
    MenuActionPayload payload,
    String sessionId,
    long renderEpoch,
    boolean inert
) {
    public MenuMetadata {
        if (menuId == null) {
            throw new IllegalArgumentException("metadata menu id is required");
        }
        route = route == null ? new MenuRoute(menuId) : route;
        if (slot < 0 || slot >= 54) {
            throw new IllegalArgumentException("metadata slot out of range");
        }
        actionKey = actionKey == null ? "" : actionKey;
        payload = payload == null ? MenuActionPayload.EMPTY : payload;
        sessionId = sessionId == null ? "" : sessionId;
    }

    public static MenuMetadata of(MenuId menuId, MenuRoute route, int slot, MenuAction action,
                                  String sessionId, long renderEpoch, boolean inert) {
        return new MenuMetadata(menuId, route, slot, MenuAction.key(action), payload(action),
            sessionId, renderEpoch, inert);
    }

    private static MenuActionPayload payload(MenuAction action) {
        return switch (action) {
            case MenuAction.DaemonCommand command -> command.body();
            case MenuAction.Select select -> select.payload();
            case MenuAction.Purchase purchase -> purchase.payload();
            default -> MenuActionPayload.EMPTY;
        };
    }
}
