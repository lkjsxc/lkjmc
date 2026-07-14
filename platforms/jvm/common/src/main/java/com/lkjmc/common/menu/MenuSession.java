package com.lkjmc.common.menu;

import java.util.ArrayDeque;
import java.util.Map;

final class MenuSession {
    private final long id;
    private final ArrayDeque<Location> history = new ArrayDeque<>();
    private String route;
    private Map<String, String> params;
    private long request;
    private long renderRevision = 1;
    private boolean pending;
    private boolean closed;

    MenuSession(long id, String route, Map<String, String> params) {
        if (id <= 0) throw new IllegalArgumentException("session id required");
        this.id = id; this.route = route; this.params = Map.copyOf(params);
    }

    void navigate(String target, Map<String, String> targetParams) {
        history.push(new Location(route, params));
        route = target; params = Map.copyOf(targetParams); renderRevision++;
    }

    void back(MenuRoute document) {
        var target = history.poll();
        route = target == null ? document.parent() : target.route();
        params = target == null ? Map.of() : target.params();
        renderRevision++;
    }

    long beginRequest() {
        if (pending) throw new IllegalStateException("request already pending");
        pending = true; return ++request;
    }

    boolean complete(long expected) {
        if (!pending || expected != request) return false;
        pending = false; renderRevision++; return true;
    }

    void close() { closed = true; pending = false; history.clear(); }
    long id() { return id; }
    String route() { return route; }
    Map<String, String> params() { return params; }
    long request() { return request; }
    long renderRevision() { return renderRevision; }
    boolean pending() { return pending; }
    boolean closed() { return closed; }

    record Location(String route, Map<String, String> params) {
        Location { params = Map.copyOf(params); }
    }
}
