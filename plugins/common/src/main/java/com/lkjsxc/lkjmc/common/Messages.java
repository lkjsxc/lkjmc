package com.lkjsxc.lkjmc.common;

import com.google.gson.*;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.regex.*;

/** Complete ID catalogs shared by Paper, Velocity and the Web. */
public final class Messages {
  private static final JsonObject REGISTRY = read("languages.json");
  private static final Map<String, JsonObject> CATALOGS = new HashMap<>();
  private static final Pattern SLOT = Pattern.compile("\\{([A-Za-z_0-9]+)\\}");

  static {
    for (JsonElement item : REGISTRY.getAsJsonArray("languages")) {
      String code = item.getAsJsonObject().get("code").getAsString();
      CATALOGS.put(code, read(code + ".json"));
    }
  }
  private Messages() {}
  private static JsonObject read(String file) {
    try (InputStream stream = Messages.class.getResourceAsStream("/locales/" + file)) {
      if (stream == null) throw new IllegalStateException("Missing language resource: " + file);
      return JsonParser.parseReader(new InputStreamReader(stream, StandardCharsets.UTF_8)).getAsJsonObject();
    } catch (IOException e) { throw new ExceptionInInitializerError(e); }
  }
  public static JsonArray languages() { return REGISTRY.getAsJsonArray("languages").deepCopy(); }
  public static String text(String language, String id, Object... values) {
    return render(language, SystemMessage.of(id, values));
  }
  public static String render(String language, JsonElement value) {
    return render(language, SystemMessage.parse(value));
  }
  public static String render(String language, SystemMessage message) {
    JsonObject catalog = CATALOGS.getOrDefault(language, CATALOGS.get(REGISTRY.get("default").getAsString()));
    if (!catalog.has(message.id())) return unknown(catalog, message);
    String template = catalog.get(message.id()).getAsString();
    Matcher matcher = SLOT.matcher(template);
    Set<String> required = new HashSet<>();
    while (matcher.find()) required.add(matcher.group(1));
    if (!required.equals(message.params().keySet())) return unknown(catalog, message);
    matcher.reset();
    StringBuilder out = new StringBuilder();
    while (matcher.find()) {
      JsonElement parameter = message.params().get(matcher.group(1));
      matcher.appendReplacement(out, Matcher.quoteReplacement(parameter == null || parameter.isJsonNull() ? "null" : parameter.getAsString()));
    }
    // One pass preserves player text containing braces or dollar/backslash characters.
    return matcher.appendTail(out).toString();
  }
  private static String unknown(JsonObject catalog, SystemMessage message) {
    JsonElement reference = message.params().get("reference");
    String id = reference != null && reference.isJsonPrimitive() ? reference.getAsString()
        : SystemMessage.unknown(message).params().get("reference").getAsString();
    return catalog.get("system.unknown").getAsString().replace("{reference}", id);
  }
  public static String error(String language, Exception exception) {
    return exception instanceof CoreClient.CoreFailure failure
        ? render(language, failure.systemMessage) : render(language, SystemMessage.unknown(exception.getMessage()));
  }
  public static String error(String language, String encoded) {
    if (encoded != null && (encoded.startsWith("text.") || encoded.startsWith("system.") || encoded.startsWith("error.")))
      return text(language, encoded);
    try { return render(language, JsonParser.parseString(encoded)); }
    catch (RuntimeException e) { return render(language, SystemMessage.unknown(encoded)); }
  }
}
