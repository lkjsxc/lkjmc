package com.lkjmc.common.workflow;

public record WorkflowView(
        WorkflowKind kind,
        WorkflowKey key,
        WorkflowPhase phase,
        long viewRevision,
        WorkflowSignal lastSignal,
        String failure) {
    public WorkflowView {
        if (kind == null || key == null || phase == null || viewRevision <= 0) {
            throw new IllegalArgumentException("invalid workflow view");
        }
        failure = failure == null ? "" : failure;
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
