package com.lkjmc.smoke;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashSet;
import java.util.Map;
import java.util.TreeMap;
import java.util.regex.Pattern;

final class SmokeText {
    private static final Pattern PAIR = Pattern.compile("\\\"((?:\\\\.|[^\\\"])*)\\\"\\s*:\\s*\\\"((?:\\\\.|[^\\\"])*)\\\"");
    private final Map<String, String> values;

    private SmokeText(Map<String, String> values) { this.values = values; }

    static SmokeText load() throws IOException {
        for (var path : candidates()) {
            if (Files.isRegularFile(path)) {
                return new SmokeText(parse(Files.readString(path, StandardCharsets.UTF_8)));
            }
        }
        try (var input = SmokeText.class.getResourceAsStream("/config/locales/en.json")) {
            if (input != null) {
                return new SmokeText(parse(new String(input.readAllBytes(), StandardCharsets.UTF_8)));
            }
        }
        throw new IllegalStateException("config/locales/en.json not found; set LKJMC_REPO_ROOT or LKJMC_LOCALE_FILE");
    }

    String key(String key) {
        var value = values.get(key);
        if (value == null) { throw new IllegalArgumentException("missing locale key " + key); }
        return plain(value);
    }

    String format(String key, Map<String, String> args) {
        var text = key(key);
        for (var entry : args.entrySet()) {
            text = text.replace("{" + entry.getKey() + "}", entry.getValue());
        }
        return text;
    }

    private static LinkedHashSet<Path> candidates() {
        var paths = new LinkedHashSet<Path>();
        addFile(paths, System.getProperty("lkjmc.locale"));
        addFile(paths, System.getenv("LKJMC_LOCALE_FILE"));
        addRoot(paths, System.getProperty("lkjmc.repo"));
        addRoot(paths, System.getenv("LKJMC_REPO_ROOT"));
        addRoot(paths, System.getenv("PWD"));
        for (Path path = Path.of(System.getProperty("user.dir", ".")).toAbsolutePath();
             path != null; path = path.getParent()) {
            addRoot(paths, path.toString());
        }
        return paths;
    }

    private static void addFile(LinkedHashSet<Path> paths, String value) {
        if (value != null && !value.isBlank()) { paths.add(Path.of(value)); }
    }

    private static void addRoot(LinkedHashSet<Path> paths, String value) {
        if (value != null && !value.isBlank()) { paths.add(Path.of(value).resolve("config/locales/en.json")); }
    }

    private static Map<String, String> parse(String json) {
        var values = new TreeMap<String, String>();
        var matcher = PAIR.matcher(json);
        while (matcher.find()) { values.put(unescape(matcher.group(1)), unescape(matcher.group(2))); }
        if (values.isEmpty()) { throw new IllegalStateException("empty locale catalog"); }
        return Map.copyOf(values);
    }

    private static String plain(String value) {
        return value.replaceAll("<[^>]+>", "");
    }

    private static String unescape(String value) {
        var out = new StringBuilder();
        for (int i = 0; i < value.length(); i++) {
            var c = value.charAt(i);
            if (c != '\\' || ++i >= value.length()) { out.append(c); continue; }
            var e = value.charAt(i);
            switch (e) {
                case 'n' -> out.append('\n');
                case 'r' -> out.append('\r');
                case 't' -> out.append('\t');
                case 'b' -> out.append('\b');
                case 'f' -> out.append('\f');
                case 'u' -> {
                    out.append((char) Integer.parseInt(value.substring(i + 1, i + 5), 16));
                    i += 4;
                }
                default -> out.append(e);
            }
        }
        return out.toString();
    }
}
