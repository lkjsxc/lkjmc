package com.lkjmc.common.ui.binding;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

final class Jsons {
    private Jsons() {}

    static JsonArray array(JsonObject object, String field, String binding) {
        var element = required(object, field, binding);
        if (!element.isJsonArray()) {
            throw fail(binding);
        }
        return element.getAsJsonArray();
    }

    static JsonObject object(JsonObject object, String field, String binding) {
        var element = required(object, field, binding);
        if (!element.isJsonObject()) {
            throw fail(binding);
        }
        return element.getAsJsonObject();
    }

    static JsonObject elementObject(JsonElement element, String binding) {
        if (element == null || !element.isJsonObject()) {
            throw fail(binding);
        }
        return element.getAsJsonObject();
    }

    static String string(JsonObject object, String field, String binding) {
        try {
            return required(object, field, binding).getAsString();
        } catch (RuntimeException error) {
            throw fail(binding);
        }
    }

    static String optionalString(JsonObject object, String field) {
        var element = object == null ? null : object.get(field);
        return element == null || element.isJsonNull() ? "" : element.getAsString();
    }

    static boolean bool(JsonObject object, String field, String binding) {
        try {
            return required(object, field, binding).getAsBoolean();
        } catch (RuntimeException error) {
            throw fail(binding);
        }
    }

    static long integer(JsonObject object, String field, String binding) {
        try {
            return required(object, field, binding).getAsLong();
        } catch (RuntimeException error) {
            throw fail(binding);
        }
    }

    static Integer nullableInt(JsonObject object, String field, String binding) {
        var element = required(object, field, binding);
        if (element.isJsonNull()) {
            return null;
        }
        try {
            return element.getAsInt();
        } catch (RuntimeException error) {
            throw fail(binding);
        }
    }

    static BindingDecodeException fail(String binding) {
        return new BindingDecodeException("menu.decode." + binding);
    }

    private static JsonElement required(JsonObject object, String field, String binding) {
        var element = object == null ? null : object.get(field);
        if (element == null) {
            throw fail(binding);
        }
        return element;
    }
}
