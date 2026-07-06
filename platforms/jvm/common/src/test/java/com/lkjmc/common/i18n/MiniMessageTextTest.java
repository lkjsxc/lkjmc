package com.lkjmc.common.i18n;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.Map;

import net.kyori.adventure.text.format.NamedTextColor;
import net.kyori.adventure.text.format.TextDecoration;
import org.junit.jupiter.api.Test;

final class MiniMessageTextTest {
    @Test
    void rendersCatalogValuesAsComponents() {
        var catalog = MessageCatalog.of(Map.of("en", Map.of("greet", "<green>Hello {name}</green>")), "en");
        var text = new MiniMessageText(catalog, new LocaleResolver("en"));

        var component = text.render("en", "greet", Map.of("name", "Alex"));

        assertEquals(NamedTextColor.GREEN, component.color());
        assertEquals("Hello Alex", text.renderPlain("en", "greet", Map.of("name", "Alex")));
    }

    @Test
    void itemNamesDisableMinecraftDefaultItalic() {
        var catalog = MessageCatalog.of(Map.of("en", Map.of("item", "<gold>Network Menu</gold>")), "en");
        var text = new MiniMessageText(catalog, new LocaleResolver("en"));

        var component = text.renderItemName("en", "item", Map.of());

        assertEquals(TextDecoration.State.FALSE, component.style().decoration(TextDecoration.ITALIC));
    }

    @Test
    void bundledCatalogValuesStrictParseAsMiniMessage() throws IOException {
        for (var locale : java.util.List.of("en", "ja")) {
            for (var entry : bundled(locale).entrySet()) {
                assertDoesNotThrow(
                    () -> MiniMessageText.parseStrict(entry.getValue()),
                    locale + ":" + entry.getKey()
                );
            }
        }
    }

    private static Map<String, String> bundled(String locale) throws IOException {
        var path = "/locales/" + locale + ".json";
        try (var input = MiniMessageTextTest.class.getResourceAsStream(path)) {
            assertNotNull(input, path);
            return MessageCatalog.parseJson(new String(input.readAllBytes(), StandardCharsets.UTF_8));
        }
    }
}
