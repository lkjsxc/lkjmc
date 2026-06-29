package com.lkjmc.common.actionbar;

public record ActionBarState(String lastDedupeKey, long lastSentAtMillis) {
    public static ActionBarState empty() {
        return new ActionBarState("", 0);
    }
}
