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
    void resolverNormalizesRegionalPlatformLocale() {
        var resolver = new LocaleResolver("en");
        assertEquals("ja", resolver.resolve(Optional.of("ja_JP")));
    }

    @Test
    void persistedLanguageOverridesPlatformLocale() {
        var resolver = new LocaleResolver("en");
        assertEquals("ja", resolver.resolve(Optional.of("ja"), Optional.of("en-US")));
    }

    @Test
    void resolverFallsBackToNetworkDefaultThenEnglish() {
        assertEquals("ja", new LocaleResolver("ja_JP").resolve(Optional.empty(), Optional.of("fr-FR")));
        assertEquals("en", new LocaleResolver("fr-FR").resolve(Optional.empty(), Optional.of("fr-FR")));
    }
}
