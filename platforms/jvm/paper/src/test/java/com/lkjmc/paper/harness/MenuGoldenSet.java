package com.lkjmc.paper.harness;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;
import com.lkjmc.common.menu.MenuAction;
import com.lkjmc.common.menu.MenuFrame;
import com.lkjmc.common.menu.MenuResult;
import com.lkjmc.paper.PaperMenuProtocolAdapter;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.Map;
import java.util.TreeMap;

final class MenuGoldenSet {
    private static final String RESOURCE = "/menu/menu-goldens.json";
    private final MenuHarnessFixtures fixtures;

    MenuGoldenSet(MenuHarnessFixtures fixtures) { this.fixtures = fixtures; }

    Map<String, String> capture() {
        var result = new TreeMap<String, String>();
        for (var route : fixtures.bundle.routes()) {
            var opened = adapter().open(1, route.id(), params(route.parameterMap()), "en");
            result.put("route/" + route.id(), hash(outcome(opened)));
        }
        for (String route : new String[]{"root", "docs-directory"}) {
            for (String locale : new String[]{"en", "ja"}) {
                var opened = adapter().open(2, route,
                        params(fixtures.bundle.route(route).parameterMap()), locale);
                result.put("matrix/" + route + "/" + locale, hash(outcome(opened)));
            }
        }
        actions(result, "en");
        actions(result, "ja");
        return result;
    }

    void verify() {
        var input = MenuGoldenSet.class.getResourceAsStream(RESOURCE);
        if (input == null) throw new AssertionError("missing menu goldens");
        var type = new TypeToken<Map<String, String>>() {}.getType();
        Map<String, String> expected = new Gson().fromJson(
                new InputStreamReader(input, StandardCharsets.UTF_8), type);
        if (!expected.equals(capture())) {
            throw new AssertionError("menu goldens changed; run updateMenuGoldens");
        }
    }

    void write(Path path) throws Exception {
        Files.createDirectories(path.getParent());
        Files.writeString(path, new GsonBuilder().setPrettyPrinting().create().toJson(capture()) + "\n");
    }

    private void actions(Map<String, String> values, String locale) {
        var adapter = adapter();
        var root = (MenuResult.Rendered) adapter.open(10, "root", Map.of(), locale);
        var navigation = root.frame().bySlot().get(15);
        values.put("action/navigate/" + locale,
                hash(outcome(adapter.click(navigation.metadata(), navigation.action()))));
        values.put("failure/stale/" + locale,
                hash(outcome(adapter.click(navigation.metadata(), navigation.action()))));
        var back = adapter.frame().bySlot().get(49);
        values.put("action/back/" + locale,
                hash(outcome(adapter.click(back.metadata(), back.action()))));

        adapter = adapter();
        root = (MenuResult.Rendered) adapter.open(11, "root", Map.of(), locale);
        var info = root.frame().bySlot().get(11);
        values.put("action/none/" + locale,
                hash(outcome(adapter.click(info.metadata(), info.action()))));
        var close = root.frame().bySlot().get(26);
        values.put("action/close/" + locale,
                hash(outcome(adapter.click(close.metadata(), close.action()))));
    }

    private PaperMenuProtocolAdapter adapter() {
        return new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer);
    }

    private static Map<String, String> params(Map<String, Boolean> fields) {
        var result = new TreeMap<String, String>();
        fields.forEach((key, required) -> {
            if (required) {
                result.put(key, switch (key) {
                    case "page" -> "0";
                    case "path" -> "docs/product/gui/README.md";
                    case "query" -> "menu";
                    default -> "sample";
                });
            }
        });
        return result;
    }

    private static String outcome(MenuResult value) {
        return switch (value) {
            case MenuResult.Rendered item -> frame(item.frame());
            case MenuResult.Failed item -> "failed|" + item.failure() + "|" + item.message();
            case MenuResult.Closed ignored -> "closed";
            case MenuResult.Ignored ignored -> "ignored";
        };
    }

    private static String frame(MenuFrame value) {
        var text = new StringBuilder(value.route()).append('|').append(value.title())
                .append('|').append(value.size());
        value.bySlot().values().stream().sorted(java.util.Comparator.comparingInt(MenuFrame.Slot::index))
                .forEach(slot -> text.append('\n').append(slot.index()).append('|').append(slot.material())
                        .append('|').append(slot.name()).append('|').append(slot.lore())
                        .append('|').append(slot.role()).append('|').append(action(slot.action())));
        return text.toString();
    }

    private static String action(MenuAction value) {
        return switch (value) {
            case MenuAction.Navigate item -> "NAVIGATE:" + item.route() + ":" + new TreeMap<>(item.params());
            case MenuAction.Simple item -> item.type().name();
        };
    }

    private static String hash(String value) {
        try {
            byte[] digest = MessageDigest.getInstance("SHA-256")
                    .digest(value.getBytes(StandardCharsets.UTF_8));
            return java.util.HexFormat.of().formatHex(digest);
        } catch (Exception failure) {
            throw new IllegalStateException(failure);
        }
    }
}
