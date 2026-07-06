package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.lkjmc.common.docs.DocBundle;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;

final class BindingTestSupport {
    private BindingTestSupport() {}

    static JsonObject json(String name) {
        try (var reader = new InputStreamReader(resource(name), StandardCharsets.UTF_8)) {
            return JsonParser.parseReader(reader).getAsJsonObject();
        } catch (Exception error) {
            throw new AssertionError(error);
        }
    }

    static BindingContext ctx() {
        return ctx(Map.of(), PermissionsView.all(), LocalData.empty());
    }

    static BindingContext ctx(Map<String, String> params) {
        return ctx(params, PermissionsView.all(), LocalData.empty());
    }

    static BindingContext ctx(Map<String, String> params, PermissionsView permissions, LocalData local) {
        return new BindingContext("player-1", "Alex", "en", params, permissions, local);
    }

    static LocalData local() {
        return new LocalData(DocBundle.load(resource("docs-bundle.json")), List.of(
            new LocalData.OnlinePlayer("player-1", "Alex", "hub"),
            new LocalData.OnlinePlayer("player-2", "Blake", "survival")));
    }

    private static java.io.InputStream resource(String name) {
        var stream = BindingTestSupport.class.getResourceAsStream("/ui/fixtures/" + name);
        if (stream == null) {
            throw new AssertionError("missing fixture " + name);
        }
        return stream;
    }
}
