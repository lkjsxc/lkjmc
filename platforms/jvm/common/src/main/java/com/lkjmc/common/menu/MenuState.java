package com.lkjmc.common.menu;

public record MenuState(MenuRoute route, MenuRouteStack routeStack, int page,
                        String sessionId, long renderEpoch) {
    public MenuState(MenuId current, int page) {
        this(new MenuRoute(current), new MenuRouteStack(java.util.List.of(new MenuRoute(current))), page, "", 0);
    }

    public MenuState {
        if (route == null) {
            throw new IllegalArgumentException("current route is required");
        }
        routeStack = routeStack == null ? new MenuRouteStack(java.util.List.of(route)) : routeStack;
        if (page < 0) {
            throw new IllegalArgumentException("page must be non-negative");
        }
        sessionId = sessionId == null ? "" : sessionId;
    }

    public MenuId current() {
        return route.id();
    }
}
