package com.lkjmc.common.ui.kernel;

import com.lkjmc.common.ui.document.MenuDocumentSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

final class UiActionDispatch {
    private UiActionDispatch() {}

    static UiStep clicked(MenuDocumentSet docs, UiModel model, UiMsg.Clicked clicked, UiIds ids) {
        var slot = UiView.frame(docs, model).slots().stream()
            .filter(value -> value.slot() == clicked.slot()).findFirst();
        if (slot.isEmpty() || slot.get().inert()) {
            return new UiStep(model, List.of());
        }
        var failure = metadataFailure(model, slot.get(), clicked);
        if (failure != null) {
            return UiUpdate.failure(model, failure);
        }
        return dispatch(docs, model, slot.get().metadata().payload(), ids);
    }

    private static MenuFailureCode metadataFailure(UiModel model, FrameSlot slot, UiMsg.Clicked clicked) {
        if (clicked.malformed() || clicked.metadata() == null || slot.metadata() == null) {
            return MenuFailureCode.UNKNOWN_METADATA;
        }
        var actual = clicked.metadata();
        if (!actual.sessionId().equals(model.sessionId())) {
            return MenuFailureCode.STALE_SESSION;
        }
        if (actual.epoch() != model.epoch()) {
            return MenuFailureCode.STALE_EPOCH;
        }
        if (!actual.route().equals(model.route()) || !actual.params().equals(model.route().params())) {
            return MenuFailureCode.ROUTE_MISMATCH;
        }
        var expected = slot.metadata();
        if (actual.slot() != slot.slot() || !actual.actionKey().equals(expected.actionKey())
            || !actual.payload().equals(expected.payload())) {
            return MenuFailureCode.UNKNOWN_METADATA;
        }
        return null;
    }

    private static UiStep dispatch(MenuDocumentSet docs, UiModel model, Map<String, String> payload, UiIds ids) {
        return switch (payload.getOrDefault("type", "none")) {
            case "none" -> new UiStep(model, List.of());
            case "open" -> UiUpdate.open(docs, model, route(payload), ids);
            case "back" -> UiUpdate.back(docs, model, ids);
            case "close" -> new UiStep(model, List.of(new UiEffect.CloseInventory()));
            case "refresh" -> UiUpdate.refresh(docs, model, List.of());
            case "command" -> new UiStep(model, List.of(new UiEffect.RunCommand(payload.getOrDefault("command", ""))));
            case "daemon" -> daemon(model, payload);
            case "input" -> prompt(model, payload);
            case "transfer" -> new UiStep(model, List.of(new UiEffect.Transfer(payload.getOrDefault("serverId", ""))));
            case "message", "disabled" -> new UiStep(model, List.of(new UiEffect.Message(
                TextRef.key(payload.getOrDefault("key", "menu.error.unknown-action")))));
            case "page" -> page(docs, model, payload.getOrDefault("direction", ""));
            default -> UiUpdate.failure(model, MenuFailureCode.UNKNOWN_METADATA);
        };
    }

    private static UiStep daemon(UiModel model, Map<String, String> payload) {
        var effect = new UiEffect.SendDaemon(DaemonRequestPlan.command(payload.getOrDefault("command", ""), body(payload)),
            TextRef.key(payload.getOrDefault("ok", "menu.action.ok")),
            TextRef.key(payload.getOrDefault("fail", "menu.action.failed")),
            Boolean.parseBoolean(payload.getOrDefault("refresh", "false")));
        return new UiStep(model, List.of(effect));
    }

    private static UiStep prompt(UiModel model, Map<String, String> payload) {
        return new UiStep(model, List.of(new UiEffect.PromptText(
            TextRef.key(payload.getOrDefault("prompt", "menu.input.prompt")),
            payload.getOrDefault("commandPrefix", ""))));
    }

    private static UiStep page(MenuDocumentSet docs, UiModel model, String direction) {
        if (!(model.phase() instanceof RoutePhase.Loaded loaded) || !(loaded.view() instanceof RouteView.ListView list)) {
            return new UiStep(model, List.of());
        }
        var size = docs.document(model.route().id()).map(UiUpdate::pageSize).orElse(1);
        var current = new Pagination(model.page(), size, list.entries().size()).clampedPage();
        var requested = "next".equals(direction) ? current + 1 : current - 1;
        var next = new Pagination(Math.max(0, requested), size, list.entries().size()).clampedPage();
        return next == current ? new UiStep(model, List.of()) : UiUpdate.phase(model, loaded, next);
    }

    private static MenuRoute route(Map<String, String> payload) {
        var params = new LinkedHashMap<String, String>();
        payload.forEach((key, value) -> {
            if (key.startsWith("param.")) {
                params.put(key.substring("param.".length()), value);
            }
        });
        return new MenuRoute(payload.getOrDefault("route", "root"), params);
    }

    private static Map<String, String> body(Map<String, String> payload) {
        var body = new LinkedHashMap<String, String>();
        payload.forEach((key, value) -> {
            if (key.startsWith("body.")) {
                body.put(key.substring("body.".length()), value);
            }
        });
        return body;
    }
}
