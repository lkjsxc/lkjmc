package com.lkjmc.common.ui.document;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

final class MenuDocumentJson {
    private MenuDocumentJson() {}

    static List<String> index(String json) {
        var menus = JsonParser.parseString(json).getAsJsonObject().getAsJsonArray("menus");
        var ids = new ArrayList<String>();
        for (var element : menus) {
            ids.add(element.getAsString());
        }
        return List.copyOf(ids);
    }

    static MenuDocument document(String json) {
        var object = JsonParser.parseString(json).getAsJsonObject();
        return new MenuDocument(
            text(object, "id"), kind(text(object, "kind")), text(object, "title"), text(object, "theme"),
            integer(object, "size"), params(array(object, "params")), nullableText(object, "parent"),
            data(object.get("data")), chrome(object.getAsJsonObject("chrome")), list(object.get("list")),
            slots(array(object, "static")), nullableText(object, "confirmation"));
    }

    private static MenuDocument.Kind kind(String value) {
        return MenuDocument.Kind.valueOf(value.toUpperCase().replace('-', '_'));
    }

    private static List<MenuDocument.Param> params(JsonArray array) {
        var values = new ArrayList<MenuDocument.Param>();
        for (var element : array) {
            var object = element.getAsJsonObject();
            values.add(new MenuDocument.Param(text(object, "name"), bool(object, "required")));
        }
        return values;
    }

    private static MenuDocument.Data data(JsonElement element) {
        if (element == null || element.isJsonNull()) {
            return null;
        }
        var object = element.getAsJsonObject();
        var source = MenuDocument.Source.valueOf(text(object, "source").toUpperCase());
        return new MenuDocument.Data(text(object, "binding"), source, strings(array(object, "commands")));
    }

    private static ChromeSpec chrome(JsonObject object) {
        if (object == null) {
            return ChromeSpec.empty();
        }
        return new ChromeSpec(nullableText(object, "info"), bool(object, "back"), bool(object, "refresh"),
            bool(object, "close"), bool(object, "mainMenu"));
    }

    private static ListGrammar list(JsonElement element) {
        if (element == null || element.isJsonNull()) {
            return null;
        }
        var object = element.getAsJsonObject();
        return new ListGrammar(text(object, "region"), nullableText(object, "reserved"),
            bool(object, "pagination"), text(object, "emptyName"), strings(array(object, "emptyLore")));
    }

    private static List<StaticSlot> slots(JsonArray array) {
        var values = new ArrayList<StaticSlot>();
        for (var element : array) {
            var object = element.getAsJsonObject();
            values.add(new StaticSlot(integer(object, "slot"), text(object, "material"), text(object, "name"),
                strings(array(object, "lore")), ItemRole.parse(text(object, "role")), action(object.getAsJsonObject("action"))));
        }
        return values;
    }

    private static DocumentAction action(JsonObject object) {
        var type = text(object, "type");
        return switch (type) {
            case "none" -> new DocumentAction.None();
            case "open" -> new DocumentAction.Open(text(object, "route"), stringMap(object.getAsJsonObject("params")));
            case "back" -> new DocumentAction.Back();
            case "close" -> new DocumentAction.Close();
            case "refresh" -> new DocumentAction.Refresh();
            case "command" -> new DocumentAction.Command(text(object, "value"));
            case "daemon" -> new DocumentAction.Daemon(text(object, "command"), stringMap(object.getAsJsonObject("body")),
                text(object, "ok"), text(object, "fail"), bool(object, "refresh"));
            case "input" -> new DocumentAction.Input(text(object, "prompt"), text(object, "commandPrefix"));
            case "message" -> new DocumentAction.Message(text(object, "key"), stringMap(object.getAsJsonObject("args")));
            default -> throw new IllegalArgumentException("unknown action type: " + type);
        };
    }

    private static JsonArray array(JsonObject object, String name) {
        var element = object == null ? null : object.get(name);
        return element == null || element.isJsonNull() ? new JsonArray() : element.getAsJsonArray();
    }

    private static List<String> strings(JsonArray array) {
        var values = new ArrayList<String>();
        for (var element : array) {
            values.add(element.getAsString());
        }
        return values;
    }

    private static Map<String, String> stringMap(JsonObject object) {
        var values = new LinkedHashMap<String, String>();
        if (object != null) {
            object.entrySet().forEach(entry -> values.put(entry.getKey(), entry.getValue().getAsString()));
        }
        return values;
    }

    private static String text(JsonObject object, String name) {
        var value = nullableText(object, name);
        if (value == null || value.isBlank()) {
            throw new IllegalArgumentException("missing string field: " + name);
        }
        return value;
    }

    private static String nullableText(JsonObject object, String name) {
        var element = object == null ? null : object.get(name);
        return element == null || element.isJsonNull() ? null : element.getAsString();
    }

    private static int integer(JsonObject object, String name) {
        return object.get(name).getAsInt();
    }

    private static boolean bool(JsonObject object, String name) {
        var element = object == null ? null : object.get(name);
        return element != null && !element.isJsonNull() && element.getAsBoolean();
    }
}
