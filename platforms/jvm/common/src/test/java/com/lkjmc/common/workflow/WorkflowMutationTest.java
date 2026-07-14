package com.lkjmc.common.workflow;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.UUID;
import org.junit.jupiter.api.Test;

final class WorkflowMutationTest {
    @Test
    void rejectsReorderedUntrustedAndIdentityMutations() {
        WorkflowKey key = key();
        WorkflowMachine machine = new WorkflowMachine(WorkflowKind.PROFILE, key);
        assertEquals(WorkflowDecision.Outcome.DENIED,
                machine.apply(key, WorkflowSignal.SAVE_ACKNOWLEDGED, true, "").outcome());
        assertFalse(machine.apply(key, WorkflowSignal.SAVE_REQUESTED, false, "").view().succeeded());
        assertEquals(WorkflowDecision.Outcome.DENIED,
                machine.apply(key, WorkflowSignal.SAVE_ACKNOWLEDGED, false, "").outcome());
        WorkflowKey stale = new WorkflowKey(key.operationId(), key.sessionId(), key.playerId(),
                key.profileRevision() - 1, key.fence(), key.correlationId());
        assertEquals(WorkflowDecision.Outcome.DENIED,
                machine.apply(stale, WorkflowSignal.SAVE_ACKNOWLEDGED, true, "").outcome());
        machine.apply(key, WorkflowSignal.SAVE_ACKNOWLEDGED, true, "");
        machine.apply(key, WorkflowSignal.LOAD_REQUESTED, false, "");
        assertTrue(machine.apply(key, WorkflowSignal.PROFILE_APPLIED, true, "").view().succeeded());
    }

    @Test
    void exactDuplicateIsStableAndChangedFailureIsDenied() {
        WorkflowKey key = key();
        WorkflowMachine machine = new WorkflowMachine(WorkflowKind.DELIVERY, key);
        WorkflowDecision first = machine.apply(key, WorkflowSignal.DELIVERY_REQUESTED, false, "");
        WorkflowDecision duplicate = machine.apply(key, WorkflowSignal.DELIVERY_REQUESTED, false, "");
        assertEquals(WorkflowDecision.Outcome.DUPLICATE, duplicate.outcome());
        assertEquals(first.view().viewRevision(), duplicate.view().viewRevision());
        machine.apply(key, WorkflowSignal.FAILED, false, "lost");
        assertEquals(WorkflowDecision.Outcome.DENIED,
                machine.apply(key, WorkflowSignal.FAILED, false, "changed").outcome());
    }

    @Test
    void historicalReplaySurvivesAdvancementAndHistoryIsBounded() {
        WorkflowKey key = key();
        WorkflowMachine machine = new WorkflowMachine(WorkflowKind.TRANSFER, key);
        machine.apply(key, WorkflowSignal.TRANSFER_REQUESTED, false, "");
        machine.apply(key, WorkflowSignal.SAVE_ACKNOWLEDGED, true, "");
        machine.apply(key, WorkflowSignal.CONNECT_REQUESTED, false, "");
        machine.apply(key, WorkflowSignal.CONNECT_COMPLETED, false, "");
        machine.apply(key, WorkflowSignal.ARRIVAL_ATTESTED, true, "");
        long revision = machine.view().viewRevision();
        for (int replay = 0; replay < 100; replay++) {
            assertEquals(WorkflowDecision.Outcome.DUPLICATE,
                    machine.apply(key, WorkflowSignal.TRANSFER_REQUESTED, false, "").outcome());
            assertEquals(revision, machine.view().viewRevision());
        }
        assertEquals(WorkflowDecision.Outcome.DENIED,
                machine.apply(key, WorkflowSignal.DELIVERY_REQUESTED, false, "").outcome());
        assertEquals(5, machine.view().replayHistory().size());

        var maximum = new ArrayList<WorkflowReplay>();
        Arrays.stream(WorkflowSignal.values()).forEach(signal ->
                maximum.add(new WorkflowReplay(signal, signal == WorkflowSignal.FAILED ? "x" : "")));
        var bounded = new WorkflowView(WorkflowKind.PROFILE, key, WorkflowPhase.FAILED, 10,
                maximum.getLast().signal(), "", maximum);
        assertEquals(WorkflowView.MAX_REPLAY_HISTORY, bounded.replayHistory().size());
        maximum.add(new WorkflowReplay(WorkflowSignal.SAVE_REQUESTED, ""));
        org.junit.jupiter.api.Assertions.assertThrows(IllegalArgumentException.class,
                () -> new WorkflowView(WorkflowKind.PROFILE, key, WorkflowPhase.FAILED, 11,
                        maximum.getLast().signal(), "", maximum));
    }

    private WorkflowKey key() {
        return new WorkflowKey(UUID.randomUUID(), UUID.randomUUID(), UUID.randomUUID(), 3, 4,
                UUID.randomUUID());
    }
}
