package com.lkjmc.common.actionbar;

import java.util.Comparator;
import java.util.List;
import java.util.Optional;

public final class ActionBarReducer {
    private ActionBarReducer() {}

    public static ActionBarDecision reduce(
        long nowMillis,
        boolean enabled,
        ActionBarState state,
        List<ActionBarFrame> frames,
        long refreshMillis
    ) {
        var current = state == null ? ActionBarState.empty() : state;
        if (!enabled) {
            return new ActionBarDecision(current, Optional.empty());
        }
        var selected = frames.stream()
            .filter(frame -> frame.activeAt(nowMillis))
            .max(Comparator.comparingInt(ActionBarFrame::priority));
        if (selected.isEmpty()) {
            return new ActionBarDecision(current, Optional.empty());
        }
        var frame = selected.get();
        if (frame.dedupeKey().equals(current.lastDedupeKey())
            && nowMillis - current.lastSentAtMillis() < refreshMillis) {
            return new ActionBarDecision(current, Optional.empty());
        }
        return new ActionBarDecision(new ActionBarState(frame.dedupeKey(), nowMillis), selected);
    }
}
