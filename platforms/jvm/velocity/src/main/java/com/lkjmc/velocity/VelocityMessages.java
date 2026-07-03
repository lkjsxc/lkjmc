package com.lkjmc.velocity;

import com.lkjmc.common.i18n.MessageCatalog;
import java.util.Map;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityMessages {
    private static final MessageCatalog CATALOG = MessageCatalog.fromResources("en", "en", "ja");

    private VelocityMessages() {}

    public static Component ok(String key) {
        return message(key, NamedTextColor.GREEN, Map.of());
    }

    public static Component error(String key) {
        return message(key, NamedTextColor.RED, Map.of());
    }

    public static Component message(String key, NamedTextColor color) {
        return message(key, color, Map.of());
    }

    public static Component message(String key, NamedTextColor color, Map<String, String> values) {
        return Component.text(render(key, values), color);
    }

    public static String render(String key, Map<String, String> values) {
        var text = CATALOG.render("en", key);
        for (var entry : values.entrySet()) {
            text = text.replace("{" + entry.getKey() + "}", entry.getValue());
        }
        return text;
    }
}
