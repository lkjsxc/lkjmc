package com.lkjsxc.lkjmc.common;

import com.google.gson.*;
import java.util.*;

/** The same language-independent wire envelope used by Core and the Web. */
public record SystemMessage(String id, Map<String, JsonElement> params) {
  public SystemMessage {
    Objects.requireNonNull(id);
    params = Collections.unmodifiableMap(new LinkedHashMap<>(params));
    if (params.values().stream().anyMatch(value -> value != null && !value.isJsonNull() && !value.isJsonPrimitive()))
      throw new IllegalArgumentException("System message parameters must be scalar");
  }

  public static SystemMessage of(String id, Object... values) {
    Map<String, JsonElement> params = new LinkedHashMap<>();
    for (int i = 0; i < values.length; i++) params.put(String.valueOf(i), new Gson().toJsonTree(values[i]));
    return new SystemMessage(id, params);
  }

  public static SystemMessage unknown(Object diagnostic) {
    String reference = "message-" + Integer.toUnsignedString(Objects.toString(diagnostic, "unknown").hashCode(), 16);
    return new SystemMessage("system.unknown", Map.of("reference", new JsonPrimitive(reference)));
  }

  public static SystemMessage parse(JsonElement value) {
    try {
      JsonObject object = value.getAsJsonObject();
      JsonObject raw = object.getAsJsonObject("params");
      Map<String, JsonElement> params = new LinkedHashMap<>();
      for (var entry : raw.entrySet()) params.put(entry.getKey(), entry.getValue());
      return new SystemMessage(object.get("id").getAsString(), params);
    } catch (RuntimeException e) {
      return unknown(value);
    }
  }

  public JsonObject json() {
    JsonObject object = new JsonObject();
    object.addProperty("id", id);
    JsonObject parameters = new JsonObject();
    params.forEach(parameters::add);
    object.add("params", parameters);
    return object;
  }

  @Override public String toString() { return json().toString(); }
}
