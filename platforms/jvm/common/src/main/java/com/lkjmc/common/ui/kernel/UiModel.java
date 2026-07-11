package com.lkjmc.common.ui.kernel;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Set;

public record UiModel(
    MenuRoute route,
    List<MenuRoute> stack,
    String sessionId,
    long epoch,
    RoutePhase phase,
    int page,
    Set<String> pendingActions,
    Map<String, String> issuedRequests,
    String playerId
) {
    public UiModel {
        route = route == null ? MenuRoute.root() : route;
        stack = normalize(stack, route);
        route = stack.get(stack.size() - 1);
        sessionId = sessionId == null ? "" : sessionId;
        phase = phase == null ? new RoutePhase.Static() : phase;
        page = Math.max(0, page);
        pendingActions = Set.copyOf(pendingActions == null ? Set.of() : pendingActions);
        issuedRequests = Map.copyOf(issuedRequests == null ? Map.of() : issuedRequests);
        playerId = playerId == null ? "" : playerId;
    }

    public UiModel(MenuRoute route, List<MenuRoute> stack, String sessionId, long epoch,
                   RoutePhase phase, int page) {
        this(route, stack, sessionId, epoch, phase, page, Set.of(), Map.of(), "");
    }

    public static UiModel root(String sessionId) {
        return root("", sessionId);
    }

    public static UiModel root(String playerId, String sessionId) {
        return new UiModel(MenuRoute.root(), List.of(MenuRoute.root()), sessionId, 0,
            new RoutePhase.Static(), 0, Set.of(), Map.of(), playerId);
    }

    public UiModel with(MenuRoute nextRoute, List<MenuRoute> nextStack, String nextSessionId,
                        long nextEpoch, RoutePhase nextPhase, int nextPage) {
        return with(nextRoute, nextStack, nextSessionId, nextEpoch, nextPhase, nextPage,
            pendingActions, Map.of());
    }

    public UiModel pending(String actionKey) {
        var next = new java.util.HashSet<>(pendingActions);
        next.add(actionKey);
        return with(route, stack, sessionId, epoch + 1, phase, page, next, Map.of());
    }

    public UiModel issued(UiRequest request) {
        var next = new java.util.HashMap<>(issuedRequests);
        next.put(request.requestId(), request.actionKey());
        return with(route, stack, sessionId, epoch, phase, page, pendingActions, next);
    }

    public UiModel complete(UiRequest request) {
        if (!pendingActions.contains(request.actionKey())) {
            return this;
        }
        var pending = new java.util.HashSet<>(pendingActions);
        var issued = new java.util.HashMap<>(issuedRequests);
        pending.remove(request.actionKey());
        issued.remove(request.requestId());
        return with(route, stack, sessionId, epoch + 1, phase, page, pending, issued);
    }

    private UiModel with(MenuRoute nextRoute, List<MenuRoute> nextStack, String nextSessionId,
                         long nextEpoch, RoutePhase nextPhase, int nextPage, Set<String> nextPending,
                         Map<String, String> nextIssued) {
        return new UiModel(nextRoute, nextStack, nextSessionId, nextEpoch, nextPhase, nextPage,
            nextPending, nextIssued, playerId);
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
