package com.lkjmc.common.sync;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.annotations.SerializedName;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.RecordComponent;
import java.lang.reflect.Type;
import java.math.BigDecimal;
import java.math.BigInteger;
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
        if (raw == String.class || raw == java.util.UUID.class || raw == Instant.class) {
            require(value.isJsonPrimitive() && value.getAsJsonPrimitive().isString());
        } else if (raw.isEnum()) {
            require(value.isJsonPrimitive() && value.getAsJsonPrimitive().isString());
            String token = value.getAsString();
            require(java.util.Arrays.stream(raw.getFields()).filter(java.lang.reflect.Field::isEnumConstant)
                    .anyMatch(field -> {
                        SerializedName name = field.getAnnotation(SerializedName.class);
                        return token.equals(name == null ? field.getName() : name.value());
                    }));
        } else if (raw == boolean.class || raw == Boolean.class) {
            require(value.isJsonPrimitive() && value.getAsJsonPrimitive().isBoolean());
        } else if (raw == int.class || raw == Integer.class
                || raw == long.class || raw == Long.class) {
            BigInteger number = integer(value);
            boolean wide = raw == long.class || raw == Long.class;
            BigInteger minimum = wide ? BigInteger.valueOf(Long.MIN_VALUE)
                    : BigInteger.valueOf(Integer.MIN_VALUE);
            BigInteger maximum = wide ? BigInteger.valueOf(Long.MAX_VALUE)
                    : BigInteger.valueOf(Integer.MAX_VALUE);
            require(number.compareTo(minimum) >= 0 && number.compareTo(maximum) <= 0);
        } else if (raw == double.class) {
            require(value.isJsonPrimitive() && value.getAsJsonPrimitive().isNumber()
                    && Double.isFinite(value.getAsDouble()));
        } else {
            throw invalid();
        }
    }

    static long integral(JsonElement value, long minimum) {
        BigInteger number = integer(value);
        require(number.compareTo(BigInteger.valueOf(minimum)) >= 0
                && number.compareTo(BigInteger.valueOf(Long.MAX_VALUE)) <= 0);
        return number.longValueExact();
    }

    private static BigInteger integer(JsonElement value) {
        require(value != null && value.isJsonPrimitive()
                && value.getAsJsonPrimitive().isNumber());
        try {
            return new BigDecimal(value.getAsJsonPrimitive().getAsString()).toBigIntegerExact();
        } catch (ArithmeticException | NumberFormatException failure) {
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
