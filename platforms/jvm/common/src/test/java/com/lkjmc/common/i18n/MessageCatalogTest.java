package com.lkjmc.common.i18n;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Map;
import java.util.Optional;

import org.junit.jupiter.api.Test;

final class MessageCatalogTest {
    @Test
    void fallsBackToEnglish() {
        var catalog = MessageCatalog.of(
            Map.of("en", Map.of("hello", "Hello"), "ja", Map.of()),
            "ja"
        );
        assertEquals("Hello", catalog.render("ja", "hello"));
    }

    @Test
    void bundledLocalesHaveSameKeys() {
        var catalog = MessageCatalog.fromResources("en", "en", "ja");
        assertTrue(catalog.hasSameKeys("en", "ja"));
    }

    @Test
    void resolverUsesPlayerLocale() {
        var resolver = new LocaleResolver("en");
        assertEquals("ja", resolver.resolve(Optional.of("ja")));
    }
}
