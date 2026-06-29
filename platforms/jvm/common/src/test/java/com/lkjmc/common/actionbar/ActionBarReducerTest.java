package com.lkjmc.common.actionbar;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.Test;

final class ActionBarReducerTest {
    @Test
    void disabledHudSuppressesOutput() {
        var frame = new ActionBarFrame(1, "Balance: 1", "balance", 0);

        var decision = ActionBarReducer.reduce(100, false, ActionBarState.empty(), List.of(frame), 1000);

        assertTrue(decision.frame().isEmpty());
    }

    @Test
    void highestPriorityActiveFrameWins() {
        var passive = new ActionBarFrame(1, "Balance", "balance", 0);
        var exchange = new ActionBarFrame(5, "+64 points", "exchange:1", 1000);

        var decision = ActionBarReducer.reduce(100, true, ActionBarState.empty(),
            List.of(passive, exchange), 1000);

        assertEquals("+64 points", decision.frame().orElseThrow().text());
    }

    @Test
    void repeatedFrameIsDedupedUntilRefresh() {
        var state = new ActionBarState("balance", 100);
        var frame = new ActionBarFrame(1, "Balance", "balance", 0);

        var quiet = ActionBarReducer.reduce(500, true, state, List.of(frame), 1000);
        var refreshed = ActionBarReducer.reduce(1200, true, state, List.of(frame), 1000);

        assertTrue(quiet.frame().isEmpty());
        assertEquals("Balance", refreshed.frame().orElseThrow().text());
    }
}
