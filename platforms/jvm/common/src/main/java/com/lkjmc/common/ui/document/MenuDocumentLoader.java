package com.lkjmc.common.ui.document;

import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.Map;

public final class MenuDocumentLoader {
    private MenuDocumentLoader() {}

    public static MenuDocumentSet fromResources() {
        var index = readResource("/menus/README.json");
        var jsonById = new LinkedHashMap<String, String>();
        for (var id : MenuDocumentJson.index(index)) {
            jsonById.put(id, readResource("/menus/" + id + ".json"));
        }
        return fromJsonStrings(jsonById);
    }

    public static MenuDocumentSet fromJsonStrings(Map<String, String> jsonById) {
        var documents = new ArrayList<MenuDocument>();
        for (var entry : jsonById.entrySet()) {
            var document = MenuDocumentJson.document(entry.getValue());
            if (!entry.getKey().equals(document.id())) {
                throw new IllegalArgumentException("menu id does not match resource: " + entry.getKey());
            }
            documents.add(document);
        }
        var set = new MenuDocumentSet(documents);
        var errors = MenuDocumentValidator.validate(set);
        if (!errors.isEmpty()) {
            throw new IllegalArgumentException("invalid menu documents: " + errors);
        }
        return set;
    }

    public static MenuDocument fromJson(String json) {
        return MenuDocumentJson.document(json);
    }

    private static String readResource(String path) {
        var stream = MenuDocumentLoader.class.getResourceAsStream(path);
        if (stream == null) {
            throw new IllegalStateException("missing menu resource: " + path);
        }
        try (var reader = new InputStreamReader(stream, StandardCharsets.UTF_8)) {
            var builder = new StringBuilder();
            var buffer = new char[2048];
            int read;
            while ((read = reader.read(buffer)) >= 0) {
                builder.append(buffer, 0, read);
            }
            return builder.toString();
        } catch (IOException error) {
            throw new IllegalStateException("failed to read menu resource: " + path, error);
        }
    }
}
