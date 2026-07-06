package com.lkjmc.common.ui.kernel;

import com.lkjmc.common.ui.document.DocumentAction;
import java.util.Map;

public record MenuMetadata(
    MenuRoute route,
    Map<String, String> params,
    int slot,
    String actionKey,
    Map<String, String> payload,
    String sessionId,
    long epoch
) {
    public MenuMetadata {
        route = route == null ? MenuRoute.root() : route;
        params = Map.copyOf(params == null ? route.params() : params);
        if (slot < 0 || slot >= 54) {
            throw new IllegalArgumentException("metadata slot out of range");
        }
        actionKey = actionKey == null ? "" : actionKey;
        payload = Map.copyOf(payload == null ? Map.of() : payload);
        sessionId = sessionId == null ? "" : sessionId;
    }

    public static MenuMetadata template(int slot, String actionKey, Map<String, String> payload) {
        return new MenuMetadata(MenuRoute.root(), Map.of(), slot, actionKey, payload, "", 0);
    }

    public static MenuMetadata template(int slot, DocumentAction action, Map<String, String> routeParams) {
        return template(slot, DocumentAction.key(action), DocumentAction.payload(action, routeParams));
    }

    public MenuMetadata stamp(MenuRoute nextRoute, String nextSessionId, long nextEpoch) {
        return new MenuMetadata(nextRoute, nextRoute.params(), slot, actionKey, payload, nextSessionId, nextEpoch);
    }
}
