package com.lkjmc.common.menu;

import java.util.Map;

public final class MenuController {
    private final MenuBundle bundle;
    private final MenuRenderer renderer;
    private MenuSession session;
    private String locale;
    private MenuSnapshotView snapshots;

    public MenuController(MenuBundle bundle, MenuRenderer renderer) {
        this.bundle = bundle; this.renderer = renderer;
    }

    public MenuResult open(long sessionId, String route, Map<String, String> params,
                           String playerLocale, MenuSnapshotView views) {
        try {
            bundle.route(route);
            session = new MenuSession(sessionId, route, params);
            locale = playerLocale; snapshots = views;
            return rendered();
        } catch (IllegalArgumentException failure) {
            return failed(MenuTypes.Failure.UNKNOWN_ROUTE);
        }
    }

    public MenuResult click(MenuFrame.Metadata metadata, MenuAction action, boolean attested) {
        if (session == null || session.closed()) return failed(MenuTypes.Failure.STALE_RENDER);
        if (metadata.session() != session.id() || !metadata.route().equals(session.route()))
            return failed(MenuTypes.Failure.STALE_RENDER);
        if (session.pending()) return failed(MenuTypes.Failure.BUSY_SESSION);
        if (metadata.renderRevision() != session.renderRevision() || metadata.request() != session.request())
            return failed(MenuTypes.Failure.STALE_RENDER);
        try {
            return switch (action) {
                case MenuAction.Navigate value -> navigate(value);
                case MenuAction.Simple value -> simple(value);
                case MenuAction.Mutation value -> mutation(value, attested);
            };
        } catch (IllegalArgumentException failure) {
            return failed(MenuTypes.Failure.MISSING_PARAMETER);
        }
    }

    public MenuResult response(long request, MenuSnapshotView views) {
        if (session == null || !session.complete(request)) return failed(MenuTypes.Failure.STALE_RESPONSE);
        snapshots = views; return rendered();
    }

    public MenuFrame frame() {
        if (session == null || session.closed()) throw new IllegalStateException("menu is closed");
        return renderer.render(session.route(), session.params(), locale, snapshots,
                session.id(), session.request(), session.renderRevision());
    }

    private MenuResult navigate(MenuAction.Navigate action) {
        bundle.route(action.route());
        session.navigate(action.route(), action.params()); return rendered();
    }

    private MenuResult simple(MenuAction.Simple action) {
        return switch (action.type()) {
            case NONE -> new MenuResult.Ignored();
            case CLOSE -> { session.close(); yield new MenuResult.Closed(); }
            case BACK -> { session.back(bundle.route(session.route())); yield rendered(); }
            case REFRESH -> new MenuResult.Pending(session.beginRequest());
            default -> failed(MenuTypes.Failure.UNKNOWN_ACTION);
        };
    }

    private MenuResult mutation(MenuAction.Mutation action, boolean attested) {
        var route = bundle.route(session.route());
        if (route.dependencies().stream().map(item -> snapshots.entry(item.domain()))
                .anyMatch(item -> item.freshness() == MenuTypes.Freshness.UNAVAILABLE))
            return failed(MenuTypes.Failure.DEPENDENCY_UNAVAILABLE);
        if (route.dependencies().stream().map(item -> snapshots.entry(item.domain()))
                .anyMatch(item -> item.freshness() == MenuTypes.Freshness.STALE))
            return failed(MenuTypes.Failure.DEPENDENCY_STALE);
        if (!snapshots.hasCurrentCapability(action.capability()))
            return failed(MenuTypes.Failure.PERMISSION_DENIED);
        if (!attested) return failed(MenuTypes.Failure.UNATTESTED);
        return failed(MenuTypes.Failure.UNSUPPORTED_OPERATION);
    }

    private MenuResult rendered() {
        try { return new MenuResult.Rendered(frame()); }
        catch (IllegalArgumentException failure) { return failed(MenuTypes.Failure.MISSING_PARAMETER); }
    }
    private MenuResult failed(MenuTypes.Failure failure) {
        return new MenuResult.Failed(failure, renderer.failure(locale == null ? "en" : locale, failure));
    }
}
