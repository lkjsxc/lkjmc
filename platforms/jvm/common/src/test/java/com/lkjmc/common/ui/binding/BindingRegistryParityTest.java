package com.lkjmc.common.ui.binding;

import static org.junit.jupiter.api.Assertions.assertEquals;

import com.lkjmc.common.ui.document.MenuDocument;
import com.lkjmc.common.ui.document.MenuDocumentLoader;
import java.util.Map;
import java.util.TreeMap;
import java.util.TreeSet;
import org.junit.jupiter.api.Test;

final class BindingRegistryParityTest {
    @Test
    void registryKeysMatchMenuDocumentBindings() {
        var docs = MenuDocumentLoader.fromResources();
        var expected = new TreeSet<String>();
        docs.documents().stream().filter(MenuDocument::bound)
            .forEach(doc -> expected.add(doc.data().binding()));

        assertEquals(expected, new TreeSet<>(BindingRegistry.standard().keys()));
    }

    @Test
    void plansMirrorDocumentSourceAndCommands() {
        var docs = MenuDocumentLoader.fromResources();
        var registry = BindingRegistry.standard();
        for (var doc : docs.documents().stream().filter(MenuDocument::bound).toList()) {
            var plan = registry.require(doc.data().binding()).plan(ctx(doc));
            assertEquals(doc.data().source().name().toLowerCase(), plan.source(), doc.id());
            assertEquals(doc.data().commands(), plan.commands(), doc.id());
            assertEquals(doc.data().commands().isEmpty() ? "" : doc.data().commands().getFirst(),
                plan.command(), doc.id());
        }
    }

    private static BindingContext ctx(MenuDocument doc) {
        var params = new TreeMap<String, String>();
        for (var param : doc.params()) {
            params.put(param.name(), value(param.name()));
        }
        if (doc.data().binding().equals("random-teleport")) {
            params.put("profileId", "overworld");
        }
        return BindingTestSupport.ctx(params);
    }

    private static String value(String name) {
        return switch (name) {
            case "home" -> "base";
            case "serverId" -> "survival";
            case "claimId" -> "claim-1";
            case "reportId" -> "report-1";
            case "kind" -> "paper";
            case "path" -> "guide/start.md";
            case "page" -> "0";
            case "query" -> "start";
            case "id" -> "first-home";
            default -> "value";
        };
    }
}
