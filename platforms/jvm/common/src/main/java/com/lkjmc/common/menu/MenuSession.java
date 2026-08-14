package com.lkjmc.common.menu;

import java.util.ArrayDeque;
import java.util.Map;

final class MenuSession {
    private final long id;
    private final ArrayDeque<Location> history = new ArrayDeque<>();
    private String route;
    private Map<String, String> params;
    private long renderRevision = 1;
    private boolean closed;

    MenuSession(long id, String route, Map<String, String> params) {
        if (id <= 0) throw new IllegalArgumentException("session id required");
        this.id = id;
        this.route = route;
        this.params = Map.copyOf(params);
    }

    void navigate(String target, Map<String, String> targetParams) {
        history.push(new Location(route, params));
        route = target;
        params = Map.copyOf(targetParams);
        renderRevision++;
    }

    void back(MenuRoute document, Map<String, String> parentParams) {
        var target = history.poll();
        route = target == null ? document.parent() : target.route();
        params = target == null ? Map.copyOf(parentParams) : target.params();
        renderRevision++;
    }

    void close() { closed = true; history.clear(); }
    long id() { return id; }
    String route() { return route; }
    Map<String, String> params() { return params; }
    long renderRevision() { return renderRevision; }
    boolean closed() { return closed; }

    record Location(String route, Map<String, String> params) {
        Location { params = Map.copyOf(params); }
    }
}
