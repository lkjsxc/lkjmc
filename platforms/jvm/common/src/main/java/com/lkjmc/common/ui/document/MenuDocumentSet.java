package com.lkjmc.common.ui.document;

import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

public record MenuDocumentSet(
    Map<String, MenuDocument> byId,
    Map<String, List<MenuDocument>> childrenByParent,
    List<MenuDocument> entrypoints
) {
    public MenuDocumentSet(Collection<MenuDocument> documents) {
        this(index(documents), children(documents), entrypoints(documents));
    }

    public MenuDocumentSet {
        byId = Map.copyOf(byId == null ? Map.of() : byId);
        childrenByParent = copyChildren(childrenByParent);
        entrypoints = List.copyOf(entrypoints == null ? List.of() : entrypoints);
    }

    public Optional<MenuDocument> document(String id) {
        return Optional.ofNullable(byId.get(id));
    }

    public MenuDocument require(String id) {
        return document(id).orElseThrow(() -> new IllegalArgumentException("unknown menu: " + id));
    }

    public Collection<MenuDocument> documents() {
        return byId.values();
    }

    public boolean contains(String id) {
        return byId.containsKey(id);
    }

    private static Map<String, MenuDocument> index(Collection<MenuDocument> documents) {
        var values = new LinkedHashMap<String, MenuDocument>();
        for (var document : documents == null ? List.<MenuDocument>of() : documents) {
            if (values.putIfAbsent(document.id(), document) != null) {
                throw new IllegalArgumentException("duplicate menu id: " + document.id());
            }
        }
        return values;
    }

    private static Map<String, List<MenuDocument>> children(Collection<MenuDocument> documents) {
        var values = new LinkedHashMap<String, List<MenuDocument>>();
        for (var document : documents == null ? List.<MenuDocument>of() : documents) {
            if (document.parent() != null && !document.parent().isBlank()) {
                values.computeIfAbsent(document.parent(), ignored -> new ArrayList<>()).add(document);
            }
        }
        return values;
    }

    private static List<MenuDocument> entrypoints(Collection<MenuDocument> documents) {
        var values = new ArrayList<MenuDocument>();
        for (var document : documents == null ? List.<MenuDocument>of() : documents) {
            if (document.parent() == null || document.parent().isBlank()) {
                values.add(document);
            }
        }
        return values;
    }

    private static Map<String, List<MenuDocument>> copyChildren(Map<String, List<MenuDocument>> source) {
        var values = new LinkedHashMap<String, List<MenuDocument>>();
        for (var entry : (source == null ? Map.<String, List<MenuDocument>>of() : source).entrySet()) {
            values.put(entry.getKey(), List.copyOf(entry.getValue()));
        }
        return Map.copyOf(values);
    }
}
