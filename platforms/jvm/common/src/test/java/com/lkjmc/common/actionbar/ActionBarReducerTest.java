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
    void repeatedFrameIsDedupedUntilRefreshWhenRequested() {
        var state = new ActionBarState("balance", 100);
        var frame = new ActionBarFrame(1, "Balance", "balance", 0);

        var quiet = ActionBarReducer.reduce(500, true, state, List.of(frame), 1000);
        var refreshed = ActionBarReducer.reduce(1200, true, state, List.of(frame), 1000);

        assertTrue(quiet.frame().isEmpty());
        assertEquals("Balance", refreshed.frame().orElseThrow().text());
    }

    @Test
    void zeroRefreshAllowsContinuousPassiveFrames() {
        var state = new ActionBarState("balance", 100);
        var frame = new ActionBarFrame(1, "Balance", "balance", 0);

        var decision = ActionBarReducer.reduce(500, true, state, List.of(frame), 0);

        assertEquals("Balance", decision.frame().orElseThrow().text());
    }

    @Test
    void playtimeNeverUsesDays() {
        assertEquals("0m", ActionBarFormatter.playtime(0));
        assertEquals("9m", ActionBarFormatter.playtime(9 * 60));
        assertEquals("1h 05m", ActionBarFormatter.playtime(65 * 60));
        assertEquals("12h 40m", ActionBarFormatter.playtime((12 * 60 + 40) * 60));
        assertEquals("123h", ActionBarFormatter.playtime(123 * 3600));
    }

    @Test
    void passiveBuilderIncludesUsefulSnapshotFields() {
        var snapshot = new ActionBarSnapshot(true, 3900, 42, "hub", 2, 10, false, 0);
        assertEquals("Play 1h 05m · Points 42 · hub · Online 2/10",
            ActionBarFrameBuilder.passiveText(snapshot));
        var local = new ActionBarSnapshot(true, 60, -1, "hub", 2, 2, false, 0);
        assertEquals("Play 1m · hub · Online 2/2", ActionBarFrameBuilder.passiveText(local));
    }
}
