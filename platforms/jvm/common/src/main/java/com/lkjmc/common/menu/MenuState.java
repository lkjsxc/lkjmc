package com.lkjmc.common.menu;

public record MenuState(MenuRoute route, MenuRouteStack routeStack, int page,
                        String sessionId, long renderEpoch) {
    public MenuState(MenuId current, int page) {
        this(new MenuRoute(current), new MenuRouteStack(java.util.List.of(new MenuRoute(current))), page, "", 0);
    }

    public MenuState {
        route = route == null ? MenuRouteStack.rootRoute() : route;
        routeStack = routeStack == null ? new MenuRouteStack(java.util.List.of(route)) : routeStack;
        if (!routeStack.last().equals(route)) {
            routeStack = routeStack.replaceTop(route);
        }
        if (page < 0) {
            throw new IllegalArgumentException("page must be non-negative");
        }
        sessionId = sessionId == null ? "" : sessionId;
    }

    public MenuId current() {
        return route.id();
    }
}
