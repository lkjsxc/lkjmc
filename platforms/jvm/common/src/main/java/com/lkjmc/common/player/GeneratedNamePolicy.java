package com.lkjmc.common.player;

import java.util.Locale;
import java.util.Set;

public final class GeneratedNamePolicy {
    private GeneratedNamePolicy() {}

    public static String nextNumbered(String base, Set<String> existing) {
        var safeBase = sanitizeBase(base);
        var used = existing == null ? Set.<String>of() : existing;
        for (int index = 1; index < 10_000; index++) {
            var candidate = index == 1 ? safeBase : safeBase + "-" + index;
            if (!used.contains(candidate)) {
                return candidate;
            }
        }
        throw new IllegalStateException("no generated name available for " + safeBase);
    }

    private static String sanitizeBase(String base) {
        var value = base == null ? "item" : base.toLowerCase(Locale.ROOT).trim();
        var safe = value.replaceAll("[^a-z0-9_-]", "-").replaceAll("-+", "-");
        safe = safe.replaceAll("^-|-$", "");
        return safe.isBlank() ? "item" : safe;
    }
}
