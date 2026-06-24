package com.lkjmc.common.i18n;

import java.util.Locale;
import java.util.Optional;

public record LocaleResolver(String networkDefault) {
    public String resolve(Optional<String> playerLocale) {
        return playerLocale
            .filter(value -> !value.isBlank())
            .map(LocaleResolver::normalize)
            .orElseGet(() -> normalize(networkDefault));
    }

    private static String normalize(String locale) {
        return Locale.forLanguageTag(locale).toLanguageTag().toLowerCase(Locale.ROOT);
    }
}
