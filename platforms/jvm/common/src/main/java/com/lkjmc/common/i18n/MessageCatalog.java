package com.lkjmc.common.i18n;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;

import com.google.gson.JsonParseException;
import com.google.gson.JsonParser;

public final class MessageCatalog {
    private final Map<String, Map<String, String>> messages;
    private final String defaultLocale;

    private MessageCatalog(Map<String, Map<String, String>> messages, String defaultLocale) {
        this.messages = Map.copyOf(messages);
        this.defaultLocale = defaultLocale;
    }

    public static MessageCatalog of(Map<String, Map<String, String>> messages, String defaultLocale) {
        var copy = new LinkedHashMap<String, Map<String, String>>();
        messages.forEach((locale, values) -> copy.put(normalize(locale), Map.copyOf(values)));
        return new MessageCatalog(copy, normalize(defaultLocale));
    }

    public static MessageCatalog fromResources(String defaultLocale, String... locales) {
        var loaded = new LinkedHashMap<String, Map<String, String>>();
        for (String locale : locales) {
            loaded.put(normalize(locale), parseResource(locale));
        }
        return of(loaded, defaultLocale);
    }

    public String render(String locale, String key) {
        return find(locale, key)
            .or(() -> find(defaultLocale, key))
            .or(() -> find("en", key))
            .orElse(key);
    }

    public Optional<String> find(String locale, String key) {
        return Optional.ofNullable(messages.get(normalize(locale))).map(values -> values.get(key));
    }

    public boolean hasSameKeys(String leftLocale, String rightLocale) {
        var left = messages.getOrDefault(normalize(leftLocale), Map.of());
        var right = messages.getOrDefault(normalize(rightLocale), Map.of());
        return left.keySet().equals(right.keySet());
    }

    private static Map<String, String> parseResource(String locale) {
        var path = "/locales/" + normalize(locale) + ".json";
        try (InputStream input = MessageCatalog.class.getResourceAsStream(path)) {
            if (input == null) {
                return Map.of();
            }
            return parseJson(new String(input.readAllBytes(), StandardCharsets.UTF_8));
        } catch (IOException error) {
            throw new IllegalStateException("failed to read locale " + locale, error);
        }
    }

    public static Map<String, String> parseJson(String json) {
        try {
            var root = JsonParser.parseString(json);
            if (!root.isJsonObject()) {
                throw new IllegalArgumentException("locale catalog must be a JSON object");
            }
            var values = new LinkedHashMap<String, String>();
            for (var entry : root.getAsJsonObject().entrySet()) {
                var value = entry.getValue();
                if (!value.isJsonPrimitive() || !value.getAsJsonPrimitive().isString()) {
                    throw new IllegalArgumentException("locale key " + entry.getKey() + " must be a string");
                }
                values.put(entry.getKey(), value.getAsString());
            }
            return Map.copyOf(values);
        } catch (JsonParseException error) {
            throw new IllegalArgumentException("invalid locale catalog JSON", error);
        }
    }

    private static String normalize(String locale) {
        return Locale.forLanguageTag(locale).toLanguageTag().toLowerCase(Locale.ROOT);
    }

}
