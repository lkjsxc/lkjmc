package com.lkjmc.paper.harness;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;
import com.lkjmc.common.menu.MenuAction;
import com.lkjmc.common.menu.MenuFrame;
import com.lkjmc.common.menu.MenuResult;
import com.lkjmc.common.menu.MenuTypes;
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
            var adapter = new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer);
            var opened = adapter.open(1, route.id(), params(route.parameterMap()), "en",
                    fixtures.views(MenuTypes.Freshness.UNAVAILABLE));
            result.put("route/" + route.id(), hash(outcome(opened)));
        }
        for (String route : new String[]{"root", "docs-directory", "shop", "claims", "settings", "admin"}) {
            for (var state : MenuTypes.Freshness.values()) {
                for (String locale : new String[]{"en", "ja"}) {
                    var adapter = new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer);
                    var opened = adapter.open(1, route, params(fixtures.bundle.route(route).parameterMap()),
                            locale, fixtures.views(state));
                    result.put("matrix/" + route + "/" + state + "/" + locale, hash(outcome(opened)));
                }
            }
        }
        actions(result, "en"); actions(result, "ja");
        return result;
    }

    void verify() {
        var input = MenuGoldenSet.class.getResourceAsStream(RESOURCE);
        if (input == null) throw new AssertionError("missing menu goldens");
        var type = new TypeToken<Map<String, String>>() {}.getType();
        Map<String, String> expected = new Gson().fromJson(
                new InputStreamReader(input, StandardCharsets.UTF_8), type);
        if (!expected.equals(capture())) throw new AssertionError("menu goldens changed; run updateMenuGoldens");
    }

    void write(Path path) throws Exception {
        Files.createDirectories(path.getParent());
        Files.writeString(path, new GsonBuilder().setPrettyPrinting().create().toJson(capture()) + "\n");
    }

    private void actions(Map<String, String> values, String locale) {
        var adapter = new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer);
        var opened = (MenuResult.Rendered) adapter.open(10, "root", Map.of(), locale,
                fixtures.views(MenuTypes.Freshness.UNAVAILABLE));
        var navigation = opened.frame().bySlot().get(19);
        values.put("action/navigate/" + locale, hash(outcome(adapter.click(
                navigation.metadata(), navigation.action(), false))));
        adapter = new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer);
        opened = (MenuResult.Rendered) adapter.open(11, "shop", Map.of(), locale,
                fixtures.views(MenuTypes.Freshness.UNAVAILABLE));
        var refresh = opened.frame().bySlot().get(50);
        var pending = (MenuResult.Pending) adapter.click(refresh.metadata(), refresh.action(), false);
        values.put("failure/busy/" + locale, hash(outcome(adapter.click(refresh.metadata(), refresh.action(), false))));
        values.put("failure/stale-response/" + locale, hash(outcome(adapter.response(
                pending.request() + 1, fixtures.views(MenuTypes.Freshness.CURRENT)))));
        adapter = new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer);
        opened = (MenuResult.Rendered) adapter.open(12, "language", Map.of(), locale,
                fixtures.views(MenuTypes.Freshness.CURRENT));
        var mutation = opened.frame().bySlot().get(20);
        values.put("failure/capability/" + locale, hash(outcome(adapter.click(
                mutation.metadata(), mutation.action(), false))));
        adapter = new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer);
        opened = (MenuResult.Rendered) adapter.open(13, "root", Map.of(), locale,
                fixtures.views(MenuTypes.Freshness.UNAVAILABLE));
        var close = opened.frame().bySlot().get(53);
        values.put("action/close/" + locale, hash(outcome(adapter.click(close.metadata(), close.action(), false))));
    }

    private static Map<String, String> params(Map<String, Boolean> fields) {
        var result = new TreeMap<String, String>();
        fields.forEach((key, required) -> {
            if (required) result.put(key, switch (key) {
                case "page" -> "0";
                case "path" -> "docs/product/gui/README.md";
                case "query" -> "menu";
                default -> "sample";
            });
        });
        return result;
    }

    private static String outcome(MenuResult value) {
        return switch (value) {
            case MenuResult.Rendered item -> frame(item.frame());
            case MenuResult.Failed item -> "failed|" + item.failure() + "|" + item.message();
            case MenuResult.Pending item -> "pending|" + item.request();
            case MenuResult.Closed ignored -> "closed";
            case MenuResult.Ignored ignored -> "ignored";
        };
    }

    private static String frame(MenuFrame value) {
        var text = new StringBuilder(value.route()).append('|').append(value.title()).append('|').append(value.size());
        value.bySlot().values().stream().sorted(java.util.Comparator.comparingInt(MenuFrame.Slot::index))
                .forEach(slot -> text.append('\n').append(slot.index()).append('|').append(slot.material())
                        .append('|').append(slot.name()).append('|').append(slot.lore())
                        .append('|').append(slot.role()).append('|').append(action(slot.action())));
        return text.toString();
    }

    private static String action(MenuAction value) {
        return switch (value) {
            case MenuAction.Navigate item -> "NAVIGATE:" + item.route() + ":" + new TreeMap<>(item.params());
            case MenuAction.Mutation item -> "MUTATION:" + item.operation() + ":" + item.capability();
            case MenuAction.Simple item -> item.type().name();
        };
    }

    private static String hash(String value) {
        try {
            byte[] digest = MessageDigest.getInstance("SHA-256").digest(value.getBytes(StandardCharsets.UTF_8));
            return java.util.HexFormat.of().formatHex(digest);
        } catch (Exception failure) { throw new IllegalStateException(failure); }
    }
}
