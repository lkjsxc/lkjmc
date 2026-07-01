package com.lkjmc.common.i18n;

import java.util.Locale;
import java.util.Optional;
import java.util.Set;

public record LocaleResolver(String networkDefault) {
    private static final Set<String> SUPPORTED = Set.of("en", "ja");

    public String resolve(Optional<String> playerLocale) {
        return resolve(Optional.empty(), playerLocale);
    }

    public String resolve(Optional<String> persistedLanguage, Optional<String> platformLocale) {
        return firstSupported(persistedLanguage)
            .or(() -> firstSupported(platformLocale))
            .or(() -> firstSupported(Optional.ofNullable(networkDefault)))
            .orElse("en");
    }

    private static Optional<String> firstSupported(Optional<String> locale) {
        return locale.filter(value -> !value.isBlank()).map(LocaleResolver::languageOnly)
            .filter(SUPPORTED::contains);
    }

    private static String languageOnly(String locale) {
        var tag = locale.replace('_', '-');
        var language = Locale.forLanguageTag(tag).getLanguage();
        return language.toLowerCase(Locale.ROOT);
    }
}
