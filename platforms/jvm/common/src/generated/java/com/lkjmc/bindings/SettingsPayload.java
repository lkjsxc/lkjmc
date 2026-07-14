package com.lkjmc.bindings;

import java.util.Map;
import java.util.UUID;

public record SettingsPayload(
        UUID playerUuid,
        String language,
        boolean menuEnabled,
        boolean hudEnabled,
        boolean tipsEnabled,
        Map<String,String> privacy
) implements DomainPayload {
    public SettingsPayload {
        java.util.Objects.requireNonNull(playerUuid, "playerUuid");
        java.util.Objects.requireNonNull(language, "language");
        if (language.isBlank()) throw new IllegalArgumentException("language");
        privacy = Map.copyOf(privacy);
    }
}
