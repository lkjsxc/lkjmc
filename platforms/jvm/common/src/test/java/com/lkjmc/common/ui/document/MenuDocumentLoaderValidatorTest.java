package com.lkjmc.common.ui.document;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

final class MenuDocumentLoaderValidatorTest {
    @Test
    void loadsEveryIndexedResource() throws Exception {
        var index = MenuDocumentJson.index(resource("/menus/README.json"));
        var loaded = MenuDocumentLoader.fromResources();

        assertEquals(index.size(), loaded.documents().size());
        for (var id : index) {
            assertTrue(loaded.contains(id), id);
            assertEquals(id, MenuDocumentLoader.fromJson(resource("/menus/" + id + ".json")).id());
        }
        assertTrue(MenuDocumentValidator.validate(loaded).isEmpty());
    }

    @Test
    void rejectsStructuralDocumentErrors() {
        var bad = new MenuDocumentSet(List.of(
            doc("root", MenuDocument.Kind.STATIC, null, null, List.of(slot(53, open("missing")))),
            doc("bad-list", MenuDocument.Kind.LIST, list("missing-region"), null, List.of()),
            doc("bad-confirm", MenuDocument.Kind.CONFIRM, null, "not-a-reason", List.of()),
            doc("static-data", MenuDocument.Kind.STATIC, null, null, List.of(), data()),
            doc("target", MenuDocument.Kind.STATIC, null, null, List.of(), null,
                List.of(new MenuDocument.Param("needed", true))),
            doc("opener", MenuDocument.Kind.STATIC, null, null,
                List.of(slot(20, new DocumentAction.Open("target", Map.of("extra", "x"))),
                    slot(20, new DocumentAction.Back())))));

        var errors = MenuDocumentValidator.validate(bad);

        assertHas(errors, MenuDocumentValidator.ChromeCollision.class);
        assertHas(errors, MenuDocumentValidator.UnknownTarget.class);
        assertHas(errors, MenuDocumentValidator.UnknownRegion.class);
        assertHas(errors, MenuDocumentValidator.InvalidConfirmation.class);
        assertHas(errors, MenuDocumentValidator.KindRule.class);
        assertHas(errors, MenuDocumentValidator.UndeclaredParam.class);
        assertHas(errors, MenuDocumentValidator.MissingOpenParam.class);
        assertHas(errors, MenuDocumentValidator.DuplicateSlot.class);
    }

    @Test
    void loaderRejectsResourceIdMismatch() {
        var json = "{\"id\":\"root\",\"kind\":\"static\",\"title\":\"menu.root.title\","
            + "\"theme\":\"root\",\"size\":54,\"params\":[],\"parent\":null,"
            + "\"chrome\":{\"close\":true},\"static\":[],\"confirmation\":null}";

        var error = org.junit.jupiter.api.Assertions.assertThrows(IllegalArgumentException.class,
            () -> MenuDocumentLoader.fromJsonStrings(Map.of("other", json)));
        assertTrue(error.getMessage().contains("resource"));
    }

    @Test
    void regionCatalogMatchesDocumentContract() {
        assertEquals(28, RegionCatalog.require("interior-28").size());
        assertEquals(21, RegionCatalog.require("interior-21").size());
        assertEquals(List.of(10, 11, 12, 13, 14, 15, 16), RegionCatalog.require("filter-row"));
        assertEquals(List.of(11, 15), RegionCatalog.require("confirm-pair"));
        assertFalse(RegionCatalog.exists("interior-22"));
    }

    private static void assertHas(List<MenuDocumentValidator.ValidationError> errors, Class<?> type) {
        assertTrue(errors.stream().anyMatch(type::isInstance), () -> "missing " + type + " in " + errors);
        assertInstanceOf(type, errors.stream().filter(type::isInstance).findFirst().orElseThrow());
    }

    private static String resource(String path) throws Exception {
        try (var stream = MenuDocumentLoaderValidatorTest.class.getResourceAsStream(path)) {
            return new String(stream.readAllBytes(), StandardCharsets.UTF_8);
        }
    }

    private static StaticSlot slot(int index, DocumentAction action) {
        return new StaticSlot(index, "STONE", "menu.root.title", List.of(), ItemRole.ACTION, action);
    }

    private static DocumentAction.Open open(String route) {
        return new DocumentAction.Open(route, Map.of());
    }

    private static ListGrammar list(String region) {
        return new ListGrammar(region, null, true, "menu.empty", List.of());
    }

    private static MenuDocument.Data data() {
        return new MenuDocument.Data("binding", MenuDocument.Source.DAEMON, List.of());
    }

    private static MenuDocument doc(String id, MenuDocument.Kind kind, ListGrammar list, String confirm,
                                    List<StaticSlot> slots) {
        return doc(id, kind, list, confirm, slots, null, List.of());
    }

    private static MenuDocument doc(String id, MenuDocument.Kind kind, ListGrammar list, String confirm,
                                    List<StaticSlot> slots, MenuDocument.Data data) {
        return doc(id, kind, list, confirm, slots, data, List.of());
    }

    private static MenuDocument doc(String id, MenuDocument.Kind kind, ListGrammar list, String confirm,
                                    List<StaticSlot> slots, MenuDocument.Data data,
                                    List<MenuDocument.Param> params) {
        return new MenuDocument(id, kind, "menu.root.title", "root", 54, params, null, data,
            new ChromeSpec("menu.root.info", false, false, true, false), list, slots, confirm);
    }
}
