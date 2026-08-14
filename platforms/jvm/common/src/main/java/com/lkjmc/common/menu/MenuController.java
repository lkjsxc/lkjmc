package com.lkjmc.common.menu;

import java.util.Map;

public final class MenuController {
    private final MenuBundle bundle;
    private final MenuRenderer renderer;
    private MenuSession session;
    private String locale;
    private MenuFrame frame;

    public MenuController(MenuBundle bundle, MenuRenderer renderer) {
        this.bundle = bundle;
        this.renderer = renderer;
    }

    public MenuResult open(long sessionId, String route, Map<String, String> params,
                           String playerLocale) {
        try {
            bundle.route(route);
        } catch (IllegalArgumentException failure) {
            return failed(MenuTypes.Failure.UNKNOWN_ROUTE);
        }
        session = new MenuSession(sessionId, route, params);
        locale = playerLocale;
        return render();
    }

    public MenuResult click(MenuFrame.Metadata metadata, MenuAction action) {
        if (session == null || session.closed()) return failed(MenuTypes.Failure.STALE_RENDER);
        if (metadata.session() != session.id() || metadata.renderRevision() != session.renderRevision()
                || !metadata.route().equals(session.route()) || metadata.action() != action.type()) {
            return failed(MenuTypes.Failure.STALE_RENDER);
        }
        return switch (action) {
            case MenuAction.Navigate value -> navigate(value);
            case MenuAction.Simple value -> simple(value);
        };
    }

    public MenuFrame frame() { return frame; }

    private MenuResult navigate(MenuAction.Navigate action) {
        final MenuRoute target;
        try {
            target = bundle.route(action.route());
        } catch (IllegalArgumentException failure) {
            return failed(MenuTypes.Failure.UNKNOWN_ROUTE);
        }
        if (missingRequired(target, action.params())) {
            return failed(MenuTypes.Failure.MISSING_PARAMETER);
        }
        session.navigate(action.route(), action.params());
        return render();
    }

    private MenuResult simple(MenuAction.Simple action) {
        return switch (action.type()) {
            case BACK -> {
                var current = bundle.route(session.route());
                session.back(current, parentParams(current));
                yield render();
            }
            case CLOSE -> {
                session.close();
                yield new MenuResult.Closed();
            }
            case NONE -> new MenuResult.Ignored();
            case NAVIGATE -> failed(MenuTypes.Failure.UNKNOWN_ACTION);
        };
    }

    private Map<String, String> parentParams(MenuRoute current) {
        if (current.parent() == null) return Map.of();
        var values = new java.util.LinkedHashMap<String, String>();
        for (var param : bundle.route(current.parent()).params()) {
            String value = session.params().get(param.name());
            if (value != null) values.put(param.name(), value);
        }
        return Map.copyOf(values);
    }

    private MenuResult render() {
        try {
            var route = bundle.route(session.route());
            if (missingRequired(route, session.params())) {
                return failed(MenuTypes.Failure.MISSING_PARAMETER);
            }
            frame = renderer.render(route, session.params(), locale,
                    session.id(), session.renderRevision());
            return new MenuResult.Rendered(frame);
        } catch (IllegalArgumentException failure) {
            return failed(MenuTypes.Failure.MISSING_PARAMETER);
        }
    }

    private static boolean missingRequired(MenuRoute route, Map<String, String> params) {
        return route.params().stream()
                .anyMatch(param -> param.required() && !params.containsKey(param.name()));
    }

    private MenuResult failed(MenuTypes.Failure failure) {
        return new MenuResult.Failed(failure, renderer.failure(locale, failure));
    }
}
