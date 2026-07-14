package com.lkjmc.common.workflow;

public record WorkflowDecision(Outcome outcome, WorkflowView view, String reason) {
    public WorkflowDecision {
        if (outcome == null || view == null || reason == null) {
            throw new IllegalArgumentException("complete decision required");
        }
    }

    public enum Outcome { APPLIED, DUPLICATE, DENIED }
}
