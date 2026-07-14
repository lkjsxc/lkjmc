package com.lkjmc.bindings;

public record ProfileSettings(
        boolean menuEnabled,
        boolean hudEnabled,
        boolean tipsEnabled,
        String privacy
) {
    public ProfileSettings {
        java.util.Objects.requireNonNull(privacy, "privacy");
        if (privacy.isBlank()) throw new IllegalArgumentException("privacy");
    }
}
