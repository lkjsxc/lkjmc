package com.lkjmc.common.workflow;

public record WorkflowReplay(WorkflowSignal signal, String failure) {
    public WorkflowReplay {
        if (signal == null) throw new IllegalArgumentException("replay signal required");
        failure = failure == null ? "" : failure;
        if (signal != WorkflowSignal.FAILED && !failure.isEmpty()) {
            throw new IllegalArgumentException("only failure replay carries detail");
        }
    }
}
