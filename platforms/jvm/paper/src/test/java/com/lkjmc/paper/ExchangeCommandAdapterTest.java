package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;

final class ExchangeCommandAdapterTest {
    @Test
    void ambiguous_commit_never_restores_items() {
        assertEquals(ExchangeCommandAdapter.Recovery.CONTAIN,
            ExchangeCommandAdapter.recovery(false, false));
    }

    @Test
    void definitive_absence_restores_only_after_reconciliation() {
        assertEquals(ExchangeCommandAdapter.Recovery.RESTORE,
            ExchangeCommandAdapter.recovery(true, false));
    }

    @Test
    void reconciled_commit_reports_points_without_inventory_refund() {
        assertEquals(ExchangeCommandAdapter.Recovery.COMMITTED,
            ExchangeCommandAdapter.recovery(false, true));
    }
}
