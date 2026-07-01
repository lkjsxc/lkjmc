package com.lkjmc.common.menu;

import java.net.URLDecoder;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.util.Collections;
import java.util.Map;
import java.util.TreeMap;
import java.util.stream.Collectors;

public record MenuActionPayload(Map<String, String> values) {
    public static final MenuActionPayload EMPTY = new MenuActionPayload(Map.of());

    public MenuActionPayload {
        values = values == null ? Map.of() : Collections.unmodifiableMap(new TreeMap<>(values));
    }

    public MenuActionPayload(String encoded) {
        this(parse(encoded));
    }

    public static MenuActionPayload of(String key, String value) {
        return new MenuActionPayload(Map.of(key, value));
    }

    public String value() {
        return values.entrySet().stream()
            .map(entry -> esc(entry.getKey()) + "=" + esc(entry.getValue()))
            .collect(Collectors.joining("&"));
    }

    private static Map<String, String> parse(String encoded) {
        if (encoded == null || encoded.isBlank()) {
            return Map.of();
        }
        var values = new TreeMap<String, String>();
        for (var part : encoded.split("&")) {
            var pair = part.split("=", 2);
            if (pair.length == 2 && !pair[0].isBlank()) {
                values.put(unesc(pair[0]), unesc(pair[1]));
            }
        }
        return values;
    }

    private static String esc(String value) {
        return URLEncoder.encode(value == null ? "" : value, StandardCharsets.UTF_8);
    }

    private static String unesc(String value) {
        return URLDecoder.decode(value, StandardCharsets.UTF_8);
    }
}
