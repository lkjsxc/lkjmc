package com.lkjmc.common.ui.kernel;

public record UiRequest(
    String playerId,
    String sessionId,
    MenuRoute route,
    long epoch,
    String requestId,
    String actionKey
) {
    public UiRequest {
        playerId = playerId == null ? "" : playerId;
        sessionId = sessionId == null ? "" : sessionId;
        route = route == null ? MenuRoute.root() : route;
        requestId = requestId == null ? "" : requestId;
        actionKey = actionKey == null ? "" : actionKey;
    }

    public static UiRequest load(UiModel model) {
        return issue(model, "load");
    }

    public static UiRequest mutation(UiModel model, String actionKey) {
        return issue(model, actionKey);
    }

    public static UiRequest none() {
        return new UiRequest("", "", MenuRoute.root(), 0, "", "");
    }

    public UiRequest forPlayer(String id) {
        return new UiRequest(id, sessionId, route, epoch, requestId, actionKey);
    }

    public boolean empty() {
        return requestId.isBlank();
    }

    public boolean matches(UiModel model) {
        return empty() || (playerId.equals(model.playerId()) && sessionId.equals(model.sessionId())
            && route.equals(model.route()) && epoch == model.epoch()
            && actionKey.equals(model.issuedRequests().get(requestId)));
    }

    private static UiRequest issue(UiModel model, String actionKey) {
        var key = actionKey == null ? "" : actionKey;
        var id = model.sessionId() + ":" + model.epoch() + ":" + model.route().id() + ":" + key;
        return new UiRequest("", model.sessionId(), model.route(), model.epoch(), id, key);
    }
}
