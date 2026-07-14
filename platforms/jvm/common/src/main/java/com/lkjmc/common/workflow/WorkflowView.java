package com.lkjmc.common.workflow;

import java.util.HashSet;
import java.util.List;

public record WorkflowView(
        WorkflowKind kind,
        WorkflowKey key,
        WorkflowPhase phase,
        long viewRevision,
        WorkflowSignal lastSignal,
        String failure,
        List<WorkflowReplay> replayHistory) {
    public static final int MAX_REPLAY_HISTORY = WorkflowSignal.values().length;

    public WorkflowView {
        if (kind == null || key == null || phase == null || viewRevision <= 0 || replayHistory == null
                || replayHistory.size() > MAX_REPLAY_HISTORY) {
            throw new IllegalArgumentException("invalid workflow view");
        }
        failure = failure == null ? "" : failure;
        replayHistory = List.copyOf(replayHistory);
        var signals = new HashSet<WorkflowSignal>();
        if (replayHistory.stream().anyMatch(item -> !signals.add(item.signal()))) {
            throw new IllegalArgumentException("duplicate workflow replay identity");
        }
        if (lastSignal == null && !replayHistory.isEmpty()
                || lastSignal != null && (replayHistory.isEmpty()
                    || replayHistory.getLast().signal() != lastSignal)) {
            throw new IllegalArgumentException("last signal/history mismatch");
        }
    }

    public boolean succeeded() {
        return phase == WorkflowPhase.PROFILE_APPLIED
                || phase == WorkflowPhase.DELIVERY_ACKNOWLEDGED
                || phase == WorkflowPhase.ARRIVED;
    }

    public boolean terminal() {
        return succeeded() || phase == WorkflowPhase.FAILED;
    }
}
