package com.lkjmc.common.sync;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.RecordComponent;
import java.lang.reflect.Type;
import java.time.Instant;
import java.util.List;
import java.util.Map;
import java.util.Set;

final class StrictRecordReader {
    private final Gson gson = new GsonBuilder()
            .registerTypeAdapter(Instant.class,
                    (com.google.gson.JsonDeserializer<Instant>) (value, type, context) ->
                            Instant.parse(value.getAsString()))
            .create();

    <T> T read(JsonElement value, Class<T> type) {
        validate(value, type);
        T decoded = gson.fromJson(value, type);
        require(decoded != null);
        return decoded;
    }

    private void validate(JsonElement value, Type type) {
        if (type instanceof ParameterizedType generic) {
            Class<?> raw = (Class<?>) generic.getRawType();
            if (raw == List.class) {
                require(value.isJsonArray() && value.getAsJsonArray().size() <= 10_000);
                value.getAsJsonArray().forEach(item -> validate(item, generic.getActualTypeArguments()[0]));
                return;
            }
            if (raw == Map.class) {
                require(value.isJsonObject() && value.getAsJsonObject().size() <= 10_000);
                value.getAsJsonObject().entrySet().forEach(item ->
                        validate(item.getValue(), generic.getActualTypeArguments()[1]));
                return;
            }
            throw invalid();
        }
        Class<?> raw = (Class<?>) type;
        if (raw.isRecord()) {
            require(value.isJsonObject());
            JsonObject object = value.getAsJsonObject();
            RecordComponent[] fields = raw.getRecordComponents();
            require(object.keySet().equals(java.util.Arrays.stream(fields)
                    .map(RecordComponent::getName).collect(java.util.stream.Collectors.toSet())));
            for (RecordComponent field : fields) {
                JsonElement item = object.get(field.getName());
                if (item.isJsonNull()) {
                    require(!field.getType().isPrimitive());
                } else {
                    validate(item, field.getGenericType());
                }
            }
            return;
        }
        if (raw == String.class || raw == java.util.UUID.class || raw == Instant.class || raw.isEnum()) {
            require(value.isJsonPrimitive() && value.getAsJsonPrimitive().isString());
        } else if (raw == boolean.class || raw == Boolean.class) {
            require(value.isJsonPrimitive() && value.getAsJsonPrimitive().isBoolean());
        } else if (raw == int.class || raw == Integer.class || raw == long.class) {
            require(value.isJsonPrimitive() && value.getAsJsonPrimitive().isNumber());
            double number = value.getAsDouble();
            require(Double.isFinite(number) && number == value.getAsLong());
            if (raw == int.class || raw == Integer.class) {
                require(number >= Integer.MIN_VALUE && number <= Integer.MAX_VALUE);
            }
        } else if (raw == double.class) {
            require(value.isJsonPrimitive() && value.getAsJsonPrimitive().isNumber()
                    && Double.isFinite(value.getAsDouble()));
        } else {
            throw invalid();
        }
    }

    static void exact(JsonObject value, String... fields) {
        require(value.keySet().equals(Set.of(fields)));
    }

    static void require(boolean condition) {
        if (!condition) throw invalid();
    }

    static IllegalArgumentException invalid() {
        return new IllegalArgumentException("invalid sync response");
    }
}
