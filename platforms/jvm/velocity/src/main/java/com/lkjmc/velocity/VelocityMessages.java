package com.lkjmc.velocity;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.i18n.MiniMessageText;
import java.util.Map;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityMessages {
    private static final MiniMessageText TEXT = new MiniMessageText(
        MessageCatalog.fromResources("en", "en", "ja"),
        new LocaleResolver("en")
    );

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
        return TEXT.render("en", key, values).colorIfAbsent(color);
    }

    public static String render(String key, Map<String, String> values) {
        return TEXT.renderPlain("en", key, values);
    }
}
