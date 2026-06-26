package com.lkjmc.common.daemon;

import com.google.gson.Gson;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.lang.reflect.Array;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;

public final class DaemonJson {
    private static final Gson GSON = new Gson();

    private DaemonJson() {}

    public static String encodeRequest(DaemonRequest request) {
        var envelope = new JsonObject();
        envelope.addProperty("requestId", request.requestId().toString());
        var actor = new JsonObject();
        actor.addProperty("kind", request.actor().kind());
        actor.addProperty("name", request.actor().name());
        envelope.add("actor", actor);
        envelope.addProperty("command", request.command());
        envelope.add("body", objectFromMap(request.body()));
        return GSON.toJson(envelope);
    }

    public static DaemonResponse decodeResponse(UUID fallbackRequestId, String json) {
        try {
            var root = JsonParser.parseString(json);
            if (!root.isJsonObject()) {
                return error(fallbackRequestId, "daemon.invalid_json", "daemon response is not an object", false);
            }
            return responseFromObject(fallbackRequestId, root.getAsJsonObject());
        } catch (RuntimeException error) {
            return error(fallbackRequestId, "daemon.invalid_json", error.getMessage(), false);
        }
    }

    public static JsonObject objectFromMap(Map<String, Object> values) {
        var object = new JsonObject();
        if (values == null) {
            return object;
        }
        values.forEach((key, value) -> object.add(key, element(value)));
        return object;
    }

    public static Optional<JsonObject> object(JsonObject object, String key) {
        return child(object, key).filter(JsonElement::isJsonObject).map(JsonElement::getAsJsonObject);
    }

    public static Optional<JsonArray> array(JsonObject object, String key) {
        return child(object, key).filter(JsonElement::isJsonArray).map(JsonElement::getAsJsonArray);
    }

    public static Optional<String> string(JsonObject object, String key) {
        return child(object, key).filter(JsonElement::isJsonPrimitive).map(JsonElement::getAsString);
    }

    public static Optional<Long> integer(JsonObject object, String key) {
        return child(object, key).filter(JsonElement::isJsonPrimitive).map(JsonElement::getAsLong);
    }

    public static Optional<Double> decimal(JsonObject object, String key) {
        return child(object, key).filter(JsonElement::isJsonPrimitive).map(JsonElement::getAsDouble);
    }

    public static boolean bool(JsonObject object, String key) {
        return child(object, key).filter(JsonElement::isJsonPrimitive).map(JsonElement::getAsBoolean).orElse(false);
    }

    public static int arraySize(JsonObject object, String key) {
        return array(object, key).map(JsonArray::size).orElse(0);
    }

    public static Optional<JsonObject> firstObject(JsonObject object, String key) {
        return array(object, key).flatMap(array -> {
            for (var element : array) {
                if (element.isJsonObject()) {
                    return Optional.of(element.getAsJsonObject());
                }
            }
            return Optional.empty();
        });
    }

    public static DaemonResponse error(UUID requestId, String code, String message, boolean retryable) {
        return new DaemonResponse(requestId, false, new JsonObject(), Optional.of(new DaemonError(code, message, retryable)));
    }

    private static DaemonResponse responseFromObject(UUID fallbackRequestId, JsonObject root) {
        var requestId = string(root, "requestId").flatMap(DaemonJson::uuid).orElse(fallbackRequestId);
        var ok = bool(root, "ok");
        var body = object(root, "body").orElseGet(JsonObject::new);
        var error = object(root, "error").map(DaemonJson::errorFromObject);
        return new DaemonResponse(requestId, ok, body, error);
    }

    private static DaemonError errorFromObject(JsonObject object) {
        return new DaemonError(
            string(object, "code").orElse("daemon.error"),
            string(object, "message").orElse("daemon error"),
            bool(object, "retryable")
        );
    }

    private static JsonElement element(Object value) {
        if (value == null) {
            return JsonNull.INSTANCE;
        }
        if (value instanceof JsonElement json) {
            return json.deepCopy();
        }
        if (value instanceof String text) {
            return primitive(text);
        }
        if (value instanceof Number number) {
            return GSON.toJsonTree(number);
        }
        if (value instanceof Boolean flag) {
            return primitive(flag);
        }
        if (value instanceof Map<?, ?> map) {
            var object = new JsonObject();
            map.forEach((key, child) -> object.add(String.valueOf(key), element(child)));
            return object;
        }
        if (value instanceof Iterable<?> iterable) {
            var array = new JsonArray();
            iterable.forEach(child -> array.add(element(child)));
            return array;
        }
        if (value.getClass().isArray()) {
            return array(value);
        }
        return primitive(value.toString());
    }

    private static JsonArray array(Object value) {
        var array = new JsonArray();
        for (var index = 0; index < Array.getLength(value); index++) {
            array.add(element(Array.get(value, index)));
        }
        return array;
    }

    private static JsonElement primitive(String value) {
        return GSON.toJsonTree(value);
    }

    private static JsonElement primitive(boolean value) {
        return GSON.toJsonTree(value);
    }

    private static Optional<JsonElement> child(JsonObject object, String key) {
        if (object == null || key == null || !object.has(key) || object.get(key).isJsonNull()) {
            return Optional.empty();
        }
        return Optional.of(object.get(key));
    }

    private static Optional<UUID> uuid(String value) {
        try {
            return Optional.of(UUID.fromString(value));
        } catch (RuntimeException error) {
            return Optional.empty();
        }
    }
}
