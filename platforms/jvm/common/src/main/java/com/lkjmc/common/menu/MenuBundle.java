package com.lkjmc.common.menu;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

public final class MenuBundle {
    private static final Set<String> ROUTE = Set.of("id", "kind", "title", "theme", "size",
            "params", "parent", "dependencies", "chrome", "slots", "dynamic", "confirmation");
    private final Map<String, MenuRoute> routes;

    private MenuBundle(Map<String, MenuRoute> routes) { this.routes = Map.copyOf(routes); }

    public static MenuBundle load(InputStream input) {
        if (input == null) throw new IllegalArgumentException("menu bundle resource missing");
        try {
            var root = JsonParser.parseReader(new InputStreamReader(input, StandardCharsets.UTF_8)).getAsJsonObject();
            exact(root, Set.of("format", "routes"));
            if (!"lkjmc-menu-bundle-v1".equals(string(root, "format"))) fail("bundle format");
            var values = new LinkedHashMap<String, MenuRoute>();
            for (var element : root.getAsJsonArray("routes")) {
                var route = parseRoute(element.getAsJsonObject());
                if (values.putIfAbsent(route.id(), route) != null) fail("duplicate route");
            }
            if (values.size() != 62 || !values.containsKey("root")) fail("route count");
            values.forEach((id, route) -> {
                if (!id.equals("root") && !values.containsKey(route.parent())) fail("parent route");
                route.slots().forEach(slot -> {
                    if (slot.action() instanceof MenuAction.Navigate action
                            && !values.containsKey(action.route())) fail("target route");
                });
            });
            return new MenuBundle(values);
        } catch (RuntimeException failure) {
            throw new IllegalArgumentException("malformed compiled menu bundle", failure);
        }
    }

    public static MenuBundle fromResource() {
        return load(MenuBundle.class.getResourceAsStream("/lkjmc-menu-bundle.json"));
    }

    public MenuRoute route(String id) {
        var value = routes.get(id);
        if (value == null) throw new IllegalArgumentException("unknown route");
        return value;
    }

    public List<MenuRoute> routes() { return routes.values().stream().sorted(
            java.util.Comparator.comparing(MenuRoute::id)).toList(); }

    private static MenuRoute parseRoute(JsonObject value) {
        exact(value, ROUTE);
        var params = new ArrayList<MenuRoute.Param>();
        for (var item : value.getAsJsonArray("params")) {
            var object = item.getAsJsonObject(); exact(object, Set.of("name", "required"));
            params.add(new MenuRoute.Param(string(object, "name"), object.get("required").getAsBoolean()));
        }
        var dependencies = new ArrayList<MenuRoute.Dependency>();
        for (var item : value.getAsJsonArray("dependencies")) {
            var object = item.getAsJsonObject(); exact(object, Set.of("domain", "scope"));
            dependencies.add(new MenuRoute.Dependency(enumValue(MenuTypes.Domain.class, string(object, "domain")),
                    enumValue(MenuTypes.Scope.class, string(object, "scope"))));
        }
        var chromeJson = value.getAsJsonObject("chrome");
        exact(chromeJson, Set.of("info", "back", "refresh", "close", "mainMenu"));
        var chrome = new MenuRoute.Chrome(nullable(chromeJson, "info"), bool(chromeJson, "back"),
                bool(chromeJson, "refresh"), bool(chromeJson, "close"), bool(chromeJson, "mainMenu"));
        var slots = new ArrayList<MenuRoute.SourceSlot>();
        for (var item : value.getAsJsonArray("slots")) slots.add(parseSlot(item.getAsJsonObject()));
        return new MenuRoute(string(value, "id"), enumValue(MenuTypes.RouteKind.class, string(value, "kind")),
                string(value, "title"), enumValue(MenuTypes.Theme.class, string(value, "theme")),
                value.get("size").getAsInt(), params, nullable(value, "parent"), dependencies, chrome, slots,
                parseDynamic(value.get("dynamic")), nullable(value, "confirmation"));
    }

    private static MenuRoute.SourceSlot parseSlot(JsonObject value) {
        exact(value, Set.of("slot", "material", "name", "lore", "role", "action"));
        return new MenuRoute.SourceSlot(value.get("slot").getAsInt(), string(value, "material"),
                string(value, "name"), strings(value.getAsJsonArray("lore")),
                enumValue(MenuTypes.Role.class, string(value, "role")), parseAction(value.getAsJsonObject("action")));
    }

    private static MenuAction parseAction(JsonObject value) {
        var type = enumValue(MenuTypes.ActionType.class, string(value, "type"));
        if (type == MenuTypes.ActionType.NAVIGATE) {
            exactEither(value, Set.of("type", "route"), Set.of("type", "route", "params"));
            var params = new LinkedHashMap<String, String>();
            if (value.has("params")) value.getAsJsonObject("params").entrySet()
                    .forEach(item -> params.put(item.getKey(), item.getValue().getAsString()));
            return new MenuAction.Navigate(string(value, "route"), params);
        }
        if (type == MenuTypes.ActionType.MUTATION) {
            exact(value, Set.of("type", "operation", "capability"));
            return new MenuAction.Mutation(enumValue(MenuTypes.Operation.class, string(value, "operation")),
                    string(value, "capability"));
        }
        exact(value, Set.of("type")); return new MenuAction.Simple(type);
    }

    private static MenuRoute.Dynamic parseDynamic(JsonElement element) {
        if (element == null || element.isJsonNull()) return null;
        var value = element.getAsJsonObject();
        var allowed = Set.of("binding", "region", "emptyName", "emptyLore");
        if (!allowed.containsAll(value.keySet()) || !value.has("binding") || !value.has("region")) fail("dynamic");
        return new MenuRoute.Dynamic(enumValue(MenuTypes.Binding.class, string(value, "binding")),
                string(value, "region"), nullable(value, "emptyName"),
                value.has("emptyLore") ? strings(value.getAsJsonArray("emptyLore")) : List.of());
    }

    private static List<String> strings(JsonArray value) {
        var result = new ArrayList<String>(); value.forEach(item -> result.add(item.getAsString())); return result;
    }
    private static String string(JsonObject value, String key) { return value.get(key).getAsString(); }
    private static String nullable(JsonObject value, String key) {
        return !value.has(key) || value.get(key).isJsonNull() ? null : value.get(key).getAsString();
    }
    private static boolean bool(JsonObject value, String key) { return value.get(key).getAsBoolean(); }
    private static <T extends Enum<T>> T enumValue(Class<T> type, String value) { return Enum.valueOf(type, value); }
    private static void exact(JsonObject value, Set<String> fields) {
        if (!value.keySet().equals(fields)) fail("members " + value.keySet());
    }
    private static void exactEither(JsonObject value, Set<String> left, Set<String> right) {
        if (!value.keySet().equals(left) && !value.keySet().equals(right)) fail("action members");
    }
    private static void fail(String message) { throw new IllegalArgumentException(message); }
}
