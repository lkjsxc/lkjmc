package com.lkjmc.common.menu;

public record MenuNavigationState(MenuRoute route, MenuRouteStack routeStack, int page,
                                  String sessionId, long renderEpoch) {
    public MenuNavigationState {
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

    public static MenuNavigationState initial() {
        return new MenuNavigationState(MenuRouteStack.rootRoute(), MenuRouteStack.root(), 0, "", 0);
    }

    public static MenuNavigationState from(MenuState state) {
        if (state == null) {
            return initial();
        }
        return new MenuNavigationState(state.route(), state.routeStack(), state.page(),
            state.sessionId(), state.renderEpoch());
    }

    public MenuNavigationState openRoot(String nextSessionId, long nextEpoch) {
        return new MenuNavigationState(MenuRouteStack.rootRoute(), MenuRouteStack.root(), 0,
            nextSessionId, nextEpoch);
    }

    public MenuNavigationState openRoute(MenuRoute target, String nextSessionId, long nextEpoch) {
        var next = target == null ? MenuRouteStack.rootRoute() : target;
        var stack = routeStack.pushDistinct(next);
        return new MenuNavigationState(stack.last(), stack, 0, nextSessionId, nextEpoch);
    }

    public MenuNavigationState back(String nextSessionId, long nextEpoch) {
        var stack = routeStack.popOrRoot();
        return new MenuNavigationState(stack.last(), stack, 0, nextSessionId, nextEpoch);
    }

    public MenuNavigationState refresh(String nextSessionId, long nextEpoch) {
        return new MenuNavigationState(route, routeStack, page, nextSessionId, nextEpoch);
    }

    public MenuNavigationState replaceDynamic(String nextSessionId, long nextEpoch) {
        return refresh(nextSessionId, nextEpoch);
    }

    public MenuState toMenuState() {
        return new MenuState(route, routeStack, page, sessionId, renderEpoch);
    }
}
