package com.lkjmc.common.i18n;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public final class MessageCatalog {
    private static final Pattern ENTRY = Pattern.compile("\"([^\"]+)\"\\s*:\\s*\"([^\"]*)\"");
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
        var values = new LinkedHashMap<String, String>();
        Matcher matcher = ENTRY.matcher(json);
        while (matcher.find()) {
            values.put(unescape(matcher.group(1)), unescape(matcher.group(2)));
        }
        return Map.copyOf(values);
    }

    private static String normalize(String locale) {
        return Locale.forLanguageTag(locale).toLanguageTag().toLowerCase(Locale.ROOT);
    }

    private static String unescape(String value) {
        return value.replace("\\\"", "\"").replace("\\n", "\n").replace("\\\\", "\\");
    }
}
