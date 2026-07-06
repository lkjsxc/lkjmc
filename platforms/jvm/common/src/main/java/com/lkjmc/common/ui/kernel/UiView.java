package com.lkjmc.common.ui.kernel;

import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.document.MenuDocument;
import com.lkjmc.common.ui.document.MenuDocumentSet;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

public final class UiView {
    private UiView() {}

    public static UiFrame frame(MenuDocumentSet docs, UiModel model) {
        var document = docs.document(model.route().id()).or(() -> docs.document("root"))
            .orElseThrow(() -> new IllegalArgumentException("root menu document is required"));
        var slots = new LinkedHashMap<Integer, FrameSlot>();
        renderPhase(slots, document, model);
        UiFrameParts.chrome(slots, document, model);
        var stamped = slots.values().stream().sorted(Comparator.comparingInt(FrameSlot::slot))
            .map(slot -> slot.stamped(model.route(), model.sessionId(), model.epoch())).toList();
        return new UiFrame(TextRef.key(document.title()), document.size(), stamped);
    }

    private static void renderPhase(Map<Integer, FrameSlot> slots, MenuDocument doc, UiModel model) {
        switch (model.phase()) {
            case RoutePhase.Static ignored -> staticSlots(slots, doc, model);
            case RoutePhase.Loading ignored -> UiFrameParts.center(slots, doc, "CLOCK", "menu.loading",
                List.of("menu.loading.lore"), ItemRole.LOADING);
            case RoutePhase.Loaded loaded -> loaded(slots, doc, model, loaded.view(), false);
            case RoutePhase.Empty ignored -> empty(slots, doc);
            case RoutePhase.Denied ignored -> UiFrameParts.center(slots, doc, "BARRIER", "menu.denied",
                List.of("menu.denied.lore"), ItemRole.DANGER);
            case RoutePhase.Stale stale -> loaded(slots, doc, model, stale.view(), true);
            case RoutePhase.Diagnostic diagnostic -> diagnostic(slots, doc, diagnostic.code());
        }
    }

    private static void loaded(Map<Integer, FrameSlot> slots, MenuDocument doc, UiModel model,
                               RouteView view, boolean stale) {
        staticSlots(slots, doc, model);
        switch (view) {
            case RouteView.ListView list -> list(slots, doc, model, list, stale);
            case RouteView.DetailView detail -> detail.slots().forEach(slot -> UiFrameParts.put(slots, slot));
            case RouteView.CustomView custom -> custom.slots().forEach(slot -> UiFrameParts.put(slots, slot));
        }
    }

    private static void list(Map<Integer, FrameSlot> slots, MenuDocument doc, UiModel model,
                             RouteView.ListView view, boolean stale) {
        view.reservedSlots().forEach(slot -> UiFrameParts.put(slots, slot));
        var region = UiFrameParts.region(doc);
        var page = new Pagination(model.page(), region.size(), view.entries().size());
        for (int index = page.firstIndex(); index < page.lastExclusive(); index++) {
            UiFrameParts.put(slots, UiFrameParts.entry(region.get(index - page.firstIndex()),
                view.entries().get(index), model.route(), stale));
        }
        if (doc.list() != null && doc.list().pagination()) {
            UiFrameParts.pageControls(slots, model.route(), page);
        }
    }

    private static void staticSlots(Map<Integer, FrameSlot> slots, MenuDocument doc, UiModel model) {
        for (var slot : doc.staticSlots()) {
            UiFrameParts.put(slots, UiFrameParts.staticSlot(slot, model.route()));
        }
    }

    private static void empty(Map<Integer, FrameSlot> slots, MenuDocument doc) {
        var name = doc.list() == null ? "menu.denied" : doc.list().emptyName();
        var lore = doc.list() == null ? List.of("menu.denied.lore") : doc.list().emptyLore();
        UiFrameParts.center(slots, doc, "BARRIER", name, lore, ItemRole.DISABLED);
    }

    private static void diagnostic(Map<Integer, FrameSlot> slots, MenuDocument doc, String code) {
        UiFrameParts.put(slots, FrameSlot.inert(UiFrameParts.centerSlot(doc), "BARRIER",
            UiFrameParts.diagnosticTitle(code), List.of(UiFrameParts.diagnosticHint(code)), ItemRole.DANGER));
    }
}
