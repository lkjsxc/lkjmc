package com.lkjmc.common.i18n;

import java.util.Map;

public record MessageRenderer(MessageCatalog catalog, LocaleResolver resolver) {
    public String render(String playerLocale, String key, Map<String, String> placeholders) {
        var message = catalog.render(resolver.resolve(java.util.Optional.ofNullable(playerLocale)), key);
        for (var entry : placeholders.entrySet()) {
            message = message.replace("{" + entry.getKey() + "}", entry.getValue());
        }
        return message;
    }
}
