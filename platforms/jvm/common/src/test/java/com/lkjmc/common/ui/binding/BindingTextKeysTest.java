package com.lkjmc.common.ui.binding;

import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.Gson;
import com.google.gson.reflect.TypeToken;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import com.lkjmc.common.ui.kernel.TextRef;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.LinkedHashSet;
import java.util.Map;
import java.util.Set;
import org.junit.jupiter.api.Test;

final class BindingTextKeysTest {
    @Test
    void emittedTextAndActionKeysExistInLocaleCatalogs() throws Exception {
        var keys = new LinkedHashSet<String>();
        var registry = BindingRegistry.standard();
        BindingDecodeFixturesTest.happyCases().forEach(arguments -> {
            var values = arguments.get();
            var result = registry.require((String) values[0]).decode(
                BindingTestSupport.json((String) values[1]), (BindingContext) values[2]);
            collect(((BindingResult.Data) result).view(), keys);
        });
        assertKeys(keys, locale("en"), "en");
        assertKeys(keys, locale("ja"), "ja");
    }

    private static void collect(RouteView view, Set<String> keys) {
        switch (view) {
            case RouteView.ListView list -> {
                list.infoLines().forEach(text -> collect(text, keys));
                list.entries().forEach(entry -> collect(entry, keys));
                list.reservedSlots().forEach(slot -> collect(slot, keys));
            }
            case RouteView.DetailView detail -> {
                detail.infoLines().forEach(text -> collect(text, keys));
                detail.slots().forEach(slot -> collect(slot, keys));
            }
            case RouteView.CustomView custom -> {
                custom.infoLines().forEach(text -> collect(text, keys));
                custom.slots().forEach(slot -> collect(slot, keys));
            }
        }
    }

    private static void collect(EntryView entry, Set<String> keys) {
        collect(entry.name(), keys);
        entry.lore().forEach(text -> collect(text, keys));
    }

    private static void collect(FrameSlot slot, Set<String> keys) {
        collect(slot.name(), keys);
        slot.lore().forEach(text -> collect(text, keys));
        if (slot.metadata() != null) {
            var payload = slot.metadata().payload();
            add(keys, payload.get("key"));
            add(keys, payload.get("ok"));
            add(keys, payload.get("fail"));
            add(keys, payload.get("prompt"));
        }
    }

    private static void collect(TextRef text, Set<String> keys) {
        if (text instanceof TextRef.Key key) {
            keys.add(key.key());
        }
    }

    private static void add(Set<String> keys, String value) {
        if (value != null && value.contains(".")) {
            keys.add(value);
        }
    }

    private static void assertKeys(Set<String> keys, Map<String, String> locale, String name) {
        var missing = keys.stream().filter(key -> !locale.containsKey(key)).toList();
        assertTrue(missing.isEmpty(), name + " missing " + missing);
    }

    private static Map<String, String> locale(String name) throws Exception {
        try (var reader = new InputStreamReader(BindingTextKeysTest.class
            .getResourceAsStream("/locales/" + name + ".json"), StandardCharsets.UTF_8)) {
            var type = new TypeToken<Map<String, String>>() {}.getType();
            return new Gson().fromJson(reader, type);
        }
    }
}
