package com.lkjmc.common.ui.kernel;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.Gson;
import com.google.gson.reflect.TypeToken;
import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.document.MenuDocument;
import com.lkjmc.common.ui.document.MenuDocumentLoader;
import com.lkjmc.common.ui.document.MenuDocumentSet;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.junit.jupiter.api.Test;

final class UiFrameBehaviorTest {
    private static final MenuDocumentSet DOCS = MenuDocumentLoader.fromResources();

    @Test
    void frameIsTotalAndAllTextKeysExist() throws Exception {
        var keys = new LinkedHashSet<String>();
        for (var document : DOCS.documents()) {
            for (var phase : phases(document)) {
                var model = model(document, phase, 0);
                var frame = UiView.frame(DOCS, model);
                assertEquals(document.size(), frame.size(), document.id());
                collect(frame.title(), keys);
                for (var slot : frame.slots()) {
                    collect(slot.name(), keys);
                    slot.lore().forEach(text -> collect(text, keys));
                    if (!slot.inert()) {
                        assertCompleteMetadata(model, slot);
                    }
                }
            }
        }
        var locale = englishLocale();
        for (var key : keys) {
            assertTrue(locale.containsKey(key), key);
        }
    }

    @Test
    void chromeUsesFixedSlots() {
        var root = UiView.frame(DOCS, model(DOCS.require("root"), new RoutePhase.Static(), 0));
        assertEquals("close", slot(root, 53).metadata().actionKey());

        var travel = UiView.frame(DOCS, model(DOCS.require("travel"), new RoutePhase.Static(), 0));
        assertEquals("open:root", slot(travel, 45).metadata().actionKey());
        assertEquals("back", slot(travel, 49).metadata().actionKey());
        assertEquals("close", slot(travel, 53).metadata().actionKey());

        var shop = UiView.frame(DOCS, model(DOCS.require("shop"), new RoutePhase.Loading(), 0));
        assertEquals("refresh", slot(shop, 50).metadata().actionKey());

        var confirm = UiView.frame(DOCS, model(DOCS.require("admin-server-delete-confirm"),
            new RoutePhase.Static(), 0));
        assertEquals("close", slot(confirm, 26).metadata().actionKey());
        assertTrue(confirm.slots().stream().noneMatch(value -> value.slot() == 53));
    }

    @Test
    void paginationRendersStableControlsAtBounds() {
        var document = DOCS.require("shop");
        var first = UiView.frame(DOCS, model(document, new RoutePhase.Loaded(listView(22)), 0));
        assertTrue(slot(first, 46).inert());
        assertFalse(slot(first, 48).inert());
        assertEquals("menu.page.info", ((TextRef.Key) slot(first, 47).name()).key());

        var second = UiView.frame(DOCS, model(document, new RoutePhase.Loaded(listView(22)), 1));
        assertFalse(slot(second, 46).inert());
        assertTrue(slot(second, 48).inert());
    }

    @Test
    void customViewCarriesPositionedSlotsAndInfoLines() {
        var action = FrameSlot.action(52, "BOOK", TextRef.key("menu.docs.title"), List.of(),
            ItemRole.NAVIGATION, new DocumentAction.Message("menu.docs.title"), Map.of());
        var view = new RouteView.CustomView("docs-file", List.of(action), List.of(TextRef.key("menu.docs.lore")));
        var frame = UiView.frame(DOCS, model(DOCS.require("docs-file"), new RoutePhase.Loaded(view), 0));

        assertEquals("message:menu.docs.title", slot(frame, 52).metadata().actionKey());
        assertEquals("menu.docs.lore", ((TextRef.Key) slot(frame, 4).lore().getFirst()).key());
    }

    private static void assertCompleteMetadata(UiModel model, FrameSlot slot) {
        assertNotNull(slot.metadata(), "metadata for slot " + slot.slot());
        assertEquals(model.route(), slot.metadata().route());
        assertEquals(model.route().params(), slot.metadata().params());
        assertEquals(model.sessionId(), slot.metadata().sessionId());
        assertEquals(model.epoch(), slot.metadata().epoch());
        assertEquals(slot.slot(), slot.metadata().slot());
        assertFalse(slot.metadata().actionKey().isBlank());
    }

    private static List<RoutePhase> phases(MenuDocument document) {
        return List.of(new RoutePhase.Static(), new RoutePhase.Loading(),
            new RoutePhase.Loaded(listView(3)), new RoutePhase.Loaded(detailView()),
            new RoutePhase.Loaded(customView()), new RoutePhase.Empty(), new RoutePhase.Denied(),
            new RoutePhase.Stale(listView(2), "daemon.http_timeout"),
            new RoutePhase.Diagnostic("menu.decode." + document.id()));
    }

    private static UiModel model(MenuDocument document, RoutePhase phase, int page) {
        var route = route(document);
        return new UiModel(route, List.of(MenuRoute.root(), route), "session", 5, phase, page);
    }

    private static MenuRoute route(MenuDocument document) {
        var params = new java.util.LinkedHashMap<String, String>();
        for (var param : document.params()) {
            if (param.required()) {
                params.put(param.name(), "value");
            }
        }
        return new MenuRoute(document.id(), params);
    }

    private static RouteView.ListView listView(int size) {
        var entries = new ArrayList<EntryView>();
        for (int i = 0; i < size; i++) {
            entries.add(new EntryView("STONE", TextRef.key("menu.root.title"), List.of(), ItemRole.ACTION,
                new DocumentAction.Message("menu.root.title")));
        }
        return new RouteView.ListView(entries, List.of(TextRef.key("menu.root.info")));
    }

    private static RouteView.DetailView detailView() {
        var slot = FrameSlot.action(20, "STONE", TextRef.key("menu.root.title"), List.of(),
            ItemRole.ACTION, new DocumentAction.Message("menu.root.title"), Map.of());
        return new RouteView.DetailView(List.of(slot), List.of(TextRef.key("menu.root.info")));
    }

    private static RouteView.CustomView customView() {
        var slot = FrameSlot.action(22, "STONE", TextRef.key("menu.root.title"), List.of(),
            ItemRole.ACTION, new DocumentAction.Message("menu.root.title"), Map.of());
        return new RouteView.CustomView("fixture", List.of(slot), List.of(TextRef.key("menu.root.info")));
    }

    private static FrameSlot slot(UiFrame frame, int index) {
        return frame.slots().stream().filter(value -> value.slot() == index).findFirst().orElseThrow();
    }

    private static void collect(TextRef text, Set<String> keys) {
        switch (text) {
            case TextRef.Key key -> keys.add(key.key());
            case TextRef.Literal literal -> throw new AssertionError("literal text in kernel: " + literal.value());
        }
    }

    private static Map<String, String> englishLocale() throws Exception {
        try (var reader = new InputStreamReader(UiFrameBehaviorTest.class.getResourceAsStream("/locales/en.json"),
            StandardCharsets.UTF_8)) {
            var type = new TypeToken<Map<String, String>>() {}.getType();
            return new Gson().fromJson(reader, type);
        }
    }
}
