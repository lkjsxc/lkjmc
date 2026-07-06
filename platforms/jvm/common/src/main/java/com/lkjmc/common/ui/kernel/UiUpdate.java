package com.lkjmc.common.ui.kernel;

import com.lkjmc.common.ui.document.MenuDocument;
import com.lkjmc.common.ui.document.MenuDocumentSet;
import com.lkjmc.common.ui.document.RegionCatalog;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

public final class UiUpdate {
    private UiUpdate() {}

    public static UiStep update(MenuDocumentSet docs, UiModel model, UiMsg msg) {
        return update(docs, model, msg, UiIds.constant(model.sessionId()));
    }

    public static UiStep update(MenuDocumentSet docs, UiModel model, UiMsg msg, UiIds ids) {
        return switch (msg) {
            case UiMsg.Open open -> open(docs, model, open.route(), ids);
            case UiMsg.Clicked clicked -> UiActionDispatch.clicked(docs, model, clicked, ids);
            case UiMsg.DataLoaded loaded -> data(docs, model, new RoutePhase.Loaded(loaded.view()), loaded.view());
            case UiMsg.DataEmpty ignored -> phase(model, new RoutePhase.Empty(), model.page());
            case UiMsg.DataDenied ignored -> phase(model, new RoutePhase.Denied(), model.page());
            case UiMsg.DataFailed failed -> phase(model, new RoutePhase.Diagnostic(failed.diagnosticCode()), model.page());
            case UiMsg.StaleAvailable stale -> data(docs, model,
                new RoutePhase.Stale(stale.view(), stale.code()), stale.view());
            case UiMsg.BackRequested ignored -> back(docs, model, ids);
            case UiMsg.RefreshRequested ignored -> refresh(docs, model, List.of());
            case UiMsg.TextSubmitted text -> text(docs, model, text);
            case UiMsg.InventoryClosed ignored -> new UiStep(model, List.of());
        };
    }

    static UiStep open(MenuDocumentSet docs, UiModel model, MenuRoute route, UiIds ids) {
        var target = route == null ? MenuRoute.root() : route;
        var document = docs.document(target.id());
        if (document.isEmpty()) {
            return phase(model, new RoutePhase.Diagnostic("menu.decode." + target.id()), model.page());
        }
        if (missingRequired(document.get(), target)) {
            var next = model.with(target, nextStack(model.stack(), target), ids.nextSessionId(), model.epoch() + 1,
                new RoutePhase.Diagnostic("menu.decode." + target.id()), 0);
            return new UiStep(next, List.of());
        }
        return openValid(document.get(), model, target, nextStack(model.stack(), target), ids.nextSessionId(), 0);
    }

    static UiStep back(MenuDocumentSet docs, UiModel model, UiIds ids) {
        if (model.stack().size() <= 1) {
            return new UiStep(model, List.of());
        }
        var stack = model.stack().subList(0, model.stack().size() - 1);
        var route = stack.get(stack.size() - 1);
        var document = docs.document(route.id()).orElse(null);
        if (document == null) {
            return phase(model, new RoutePhase.Diagnostic("menu.decode." + route.id()), model.page());
        }
        return openValid(document, model, route, stack, ids.nextSessionId(), 0);
    }

    static UiStep refresh(MenuDocumentSet docs, UiModel model, List<UiEffect> before) {
        var document = docs.document(model.route().id()).orElse(null);
        if (document == null) {
            return addEffects(phase(model, new RoutePhase.Diagnostic("menu.decode." + model.route().id()), model.page()), before);
        }
        var effects = new ArrayList<>(before);
        var phase = document.bound() ? new RoutePhase.Loading() : new RoutePhase.Static();
        if (document.bound()) {
            effects.add(load(document, model.route()));
        }
        var next = model.with(model.route(), model.stack(), model.sessionId(), model.epoch() + 1, phase, model.page());
        return new UiStep(next, effects);
    }

    static UiStep phase(UiModel model, RoutePhase phase, int page) {
        return new UiStep(model.with(model.route(), model.stack(), model.sessionId(), model.epoch() + 1, phase, page),
            List.of());
    }

    static UiStep failure(UiModel model, MenuFailureCode failure) {
        return new UiStep(model, List.of(new UiEffect.Message(TextRef.key(failure.messageKey()))));
    }

    static int pageSize(MenuDocument document) {
        if (document.list() != null && RegionCatalog.exists(document.list().region())) {
            return RegionCatalog.require(document.list().region()).size();
        }
        return 1;
    }

    private static UiStep openValid(MenuDocument document, UiModel model, MenuRoute route,
                                    List<MenuRoute> stack, String sessionId, int page) {
        var phase = document.bound() ? new RoutePhase.Loading() : new RoutePhase.Static();
        var next = model.with(route, stack, sessionId, model.epoch() + 1, phase, page);
        return new UiStep(next, document.bound() ? List.of(load(document, route)) : List.of());
    }

    private static UiStep text(MenuDocumentSet docs, UiModel model, UiMsg.TextSubmitted text) {
        return refresh(docs, model, List.of(new UiEffect.RunCommand(join(text.commandPrefix(), text.text()))));
    }

    private static UiStep data(MenuDocumentSet docs, UiModel model, RoutePhase phase, RouteView view) {
        return phase(model, phase, clampedPage(docs, model, view));
    }

    private static UiEffect.LoadData load(MenuDocument document, MenuRoute route) {
        return new UiEffect.LoadData(DaemonRequestPlan.load(document.data().binding(),
            document.data().source().name().toLowerCase(Locale.ROOT), route, document.data().commands()));
    }

    private static List<MenuRoute> nextStack(List<MenuRoute> stack, MenuRoute route) {
        if (route.isRoot()) {
            return List.of(MenuRoute.root());
        }
        var next = new ArrayList<>(stack == null || stack.isEmpty() ? List.of(MenuRoute.root()) : stack);
        var top = next.get(next.size() - 1);
        if (top.id().equals(route.id())) {
            next.set(next.size() - 1, route);
        } else {
            next.add(route);
        }
        return List.copyOf(next);
    }

    private static boolean missingRequired(MenuDocument document, MenuRoute route) {
        return document.params().stream().anyMatch(param -> param.required() && !route.params().containsKey(param.name()));
    }

    private static int clampedPage(MenuDocumentSet docs, UiModel model, RouteView view) {
        if (view instanceof RouteView.ListView list) {
            return new Pagination(model.page(), docs.document(model.route().id()).map(UiUpdate::pageSize).orElse(1),
                list.entries().size()).clampedPage();
        }
        return model.page();
    }

    private static UiStep addEffects(UiStep step, List<UiEffect> before) {
        var effects = new ArrayList<>(before);
        effects.addAll(step.effects());
        return new UiStep(step.model(), effects);
    }

    private static String join(String prefix, String text) {
        var first = prefix == null ? "" : prefix.stripTrailing();
        var second = text == null ? "" : text.strip();
        return first.isBlank() ? second : first + " " + second;
    }
}
