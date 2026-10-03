package com.lkjsxc.lkjmc.common;

import com.google.gson.*;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.*;

/** English message IDs, optional language catalogs, and an explicit English fallback. */
public final class Messages {
  private static final JsonObject REGISTRY = read("languages.json");
  private static final Map<String, JsonObject> CATALOGS = new HashMap<>();
  private static final Map<String, String> ERROR_KEYS = new HashMap<>();

  static {
    for (JsonElement item : REGISTRY.getAsJsonArray("languages")) {
      String code = item.getAsJsonObject().get("code").getAsString();
      if (!code.equals("en")) {
        JsonObject catalog = read(code + ".json");
        CATALOGS.put(code, catalog);
        for (var entry : catalog.entrySet())
          ERROR_KEYS.put(entry.getValue().getAsString(), entry.getKey());
      }
    }
  }

  private Messages() {}

  private static JsonObject read(String file) {
    try (InputStream stream = Messages.class.getResourceAsStream("/locales/" + file)) {
      if (stream == null) throw new IllegalStateException("Missing language resource: " + file);
      return JsonParser.parseReader(new InputStreamReader(stream, StandardCharsets.UTF_8))
          .getAsJsonObject();
    } catch (IOException e) {
      throw new ExceptionInInitializerError(e);
    }
  }

  public static JsonArray languages() {
    return REGISTRY.getAsJsonArray("languages").deepCopy();
  }

  public static String text(String language, String key, Object... values) {
    JsonObject catalog = CATALOGS.get(language);
    String result = catalog != null && catalog.has(key) ? catalog.get(key).getAsString() : key;
    // A single pass prevents player-supplied placeholders from being interpreted.
    var matcher = java.util.regex.Pattern.compile("\\{(\\d+)\\}").matcher(result);
    StringBuilder out = new StringBuilder();
    while (matcher.find()) {
      int index = Integer.parseInt(matcher.group(1));
      matcher.appendReplacement(
          out,
          java.util.regex.Matcher.quoteReplacement(
              index < values.length ? String.valueOf(values[index]) : matcher.group()));
    }
    return matcher.appendTail(out).toString();
  }

  public static String error(String language, String message) {
    String key = ERROR_KEYS.getOrDefault(message, message);
    JsonObject catalog = CATALOGS.get(language);
    if (catalog == null || catalog.has(key)) return text(language, key);
    for (var entry : catalog.entrySet()) {
      String template = entry.getKey();
      var slots = java.util.regex.Pattern.compile("\\{[A-Za-z_0-9]*\\}").matcher(template);
      List<String> names = new ArrayList<>();
      StringBuilder pattern = new StringBuilder("^");
      int end = 0;
      while (slots.find()) {
        pattern
            .append(java.util.regex.Pattern.quote(template.substring(end, slots.start())))
            .append("(.+?)");
        names.add(slots.group());
        end = slots.end();
      }
      if (names.isEmpty()) continue;
      pattern.append(java.util.regex.Pattern.quote(template.substring(end))).append("$");
      var match =
          java.util.regex.Pattern.compile(pattern.toString(), java.util.regex.Pattern.DOTALL)
              .matcher(message);
      if (match.matches()) {
        var replacements =
            java.util.regex.Pattern.compile("\\{[A-Za-z_0-9]*\\}")
                .matcher(entry.getValue().getAsString());
        StringBuilder output = new StringBuilder();
        while (replacements.find()) {
          int index = names.indexOf(replacements.group());
          replacements.appendReplacement(
              output,
              java.util.regex.Matcher.quoteReplacement(
                  index >= 0 ? match.group(index + 1) : replacements.group()));
        }
        return replacements.appendTail(output).toString();
      }
    }
    return key;
  }
}
