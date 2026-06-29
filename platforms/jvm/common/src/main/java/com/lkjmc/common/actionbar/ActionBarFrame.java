package com.lkjmc.common.actionbar;

public record ActionBarFrame(int priority, String text, String dedupeKey, long expiresAtMillis) {
    public ActionBarFrame {
        if (text == null || text.isBlank()) {
            throw new IllegalArgumentException("text is required");
        }
        if (dedupeKey == null || dedupeKey.isBlank()) {
            throw new IllegalArgumentException("dedupe key is required");
        }
    }

    public boolean activeAt(long nowMillis) {
        return expiresAtMillis <= 0 || expiresAtMillis > nowMillis;
    }
}
