package com.lkjmc.common.i18n;

import java.util.Map;
import java.util.Optional;

import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.TextComponent;
import net.kyori.adventure.text.format.TextDecoration;
import net.kyori.adventure.text.minimessage.MiniMessage;

public final class MiniMessageText {
    private static final MiniMessage MINI_MESSAGE = MiniMessage.builder().strict(true).build();
    private static final String NOT_ITALIC = "<!italic>";
    private static final String NOT_ITALIC_CLOSE = "</!italic>";
    private final MessageCatalog catalog;
    private final LocaleResolver resolver;

    public MiniMessageText(MessageCatalog catalog, LocaleResolver resolver) {
        this.catalog = catalog;
        this.resolver = resolver;
    }

    public Component render(String locale, String key) {
        return render(locale, key, Map.of());
    }

    public Component render(String locale, String key, Map<String, String> placeholders) {
        return parseStrict(renderMarkup(locale, key, placeholders));
    }

    public Component renderItemName(String locale, String key, Map<String, String> placeholders) {
        return parseStrict(NOT_ITALIC + renderMarkup(locale, key, placeholders) + NOT_ITALIC_CLOSE)
            .decoration(TextDecoration.ITALIC, false);
    }

    public String renderMarkup(String locale, String key, Map<String, String> placeholders) {
        var message = catalog.render(resolver.resolve(Optional.ofNullable(locale)), key);
        for (var entry : placeholders.entrySet()) {
            message = message.replace("{" + entry.getKey() + "}", entry.getValue());
        }
        return message;
    }

    public String renderPlain(String locale, String key, Map<String, String> placeholders) {
        return plain(render(locale, key, placeholders));
    }

    public static Component parseStrict(String message) {
        return MINI_MESSAGE.deserialize(message);
    }

    private static String plain(Component component) {
        var output = new StringBuilder();
        appendPlain(component, output);
        return output.toString();
    }

    private static void appendPlain(Component component, StringBuilder output) {
        if (component instanceof TextComponent text) {
            output.append(text.content());
        }
        component.children().forEach(child -> appendPlain(child, output));
    }
}
