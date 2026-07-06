package com.lkjmc.common.ui.kernel;

import java.util.ArrayList;
import java.util.List;

public record UiModel(
    MenuRoute route,
    List<MenuRoute> stack,
    String sessionId,
    long epoch,
    RoutePhase phase,
    int page
) {
    public UiModel {
        route = route == null ? MenuRoute.root() : route;
        stack = normalize(stack, route);
        route = stack.get(stack.size() - 1);
        sessionId = sessionId == null ? "" : sessionId;
        phase = phase == null ? new RoutePhase.Static() : phase;
        page = Math.max(0, page);
    }

    public static UiModel root(String sessionId) {
        return new UiModel(MenuRoute.root(), List.of(MenuRoute.root()), sessionId, 0,
            new RoutePhase.Static(), 0);
    }

    public UiModel with(MenuRoute nextRoute, List<MenuRoute> nextStack, String nextSessionId,
                        long nextEpoch, RoutePhase nextPhase, int nextPage) {
        return new UiModel(nextRoute, nextStack, nextSessionId, nextEpoch, nextPhase, nextPage);
    }

    private static List<MenuRoute> normalize(List<MenuRoute> input, MenuRoute route) {
        var values = new ArrayList<MenuRoute>();
        if (input != null) {
            values.addAll(input);
        }
        if (values.isEmpty() || !values.get(0).isRoot()) {
            values.add(0, MenuRoute.root());
        }
        if (route.isRoot()) {
            return List.of(MenuRoute.root());
        }
        if (values.size() == 1 || !values.get(values.size() - 1).equals(route)) {
            values.add(route);
        }
        return List.copyOf(values);
    }
}
