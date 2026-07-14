package com.lkjmc.common.workflow;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

public final class WorkflowMachine {
    private static final Map<WorkflowPhase, Map<WorkflowSignal, WorkflowPhase>> NEXT = Map.of(
            WorkflowPhase.CREATED, Map.of(
                    WorkflowSignal.SAVE_REQUESTED, WorkflowPhase.SAVE_REQUESTED,
                    WorkflowSignal.DELIVERY_REQUESTED, WorkflowPhase.DELIVERY_REQUESTED,
                    WorkflowSignal.TRANSFER_REQUESTED, WorkflowPhase.TRANSFER_REQUESTED),
            WorkflowPhase.SAVE_REQUESTED, Map.of(
                    WorkflowSignal.SAVE_ACKNOWLEDGED, WorkflowPhase.SAVE_ACKNOWLEDGED),
            WorkflowPhase.SAVE_ACKNOWLEDGED, Map.of(
                    WorkflowSignal.LOAD_REQUESTED, WorkflowPhase.LOAD_REQUESTED,
                    WorkflowSignal.CONNECT_REQUESTED, WorkflowPhase.CONNECT_REQUESTED),
            WorkflowPhase.LOAD_REQUESTED, Map.of(
                    WorkflowSignal.PROFILE_APPLIED, WorkflowPhase.PROFILE_APPLIED),
            WorkflowPhase.DELIVERY_REQUESTED, Map.of(
                    WorkflowSignal.DELIVERY_ACKNOWLEDGED, WorkflowPhase.DELIVERY_ACKNOWLEDGED),
            WorkflowPhase.TRANSFER_REQUESTED, Map.of(
                    WorkflowSignal.SAVE_ACKNOWLEDGED, WorkflowPhase.SAVE_ACKNOWLEDGED),
            WorkflowPhase.CONNECT_REQUESTED, Map.of(
                    WorkflowSignal.CONNECT_COMPLETED, WorkflowPhase.CONNECTED),
            WorkflowPhase.CONNECTED, Map.of(
                    WorkflowSignal.ARRIVAL_ATTESTED, WorkflowPhase.ARRIVED));

    private WorkflowView view;

    public WorkflowMachine(WorkflowKind kind, WorkflowKey key) {
        view = new WorkflowView(kind, key, WorkflowPhase.CREATED, 1, null, "", List.of());
    }

    public WorkflowMachine(WorkflowView restored) {
        view = restored;
    }

    public synchronized WorkflowDecision apply(
            WorkflowKey supplied,
            WorkflowSignal signal,
            boolean trustedObservation,
            String failure) {
        if (!view.key().equals(supplied)) return denied("workflow identity mismatch");
        WorkflowReplay replay = view.replayHistory().stream()
                .filter(item -> item.signal() == signal).findFirst().orElse(null);
        if (replay != null) {
            if (replay.failure().equals(replayFailure(signal, failure))) return duplicate();
            return denied("changed replay");
        }
        if (view.terminal()) return denied("workflow is terminal");
        if (signal == WorkflowSignal.FAILED) return advance(WorkflowPhase.FAILED, signal, failure);
        WorkflowPhase target = NEXT.getOrDefault(view.phase(), Map.of()).get(signal);
        if (target == null || !allowedForKind(signal)) return denied("reordered or skipped transition");
        if (requiresProof(signal) && !trustedObservation) return denied("trusted observation required");
        return advance(target, signal, "");
    }

    public synchronized WorkflowView view() {
        return view;
    }

    private boolean allowedForKind(WorkflowSignal signal) {
        return switch (view.kind()) {
            case PROFILE -> signal == WorkflowSignal.SAVE_REQUESTED
                    || signal == WorkflowSignal.SAVE_ACKNOWLEDGED
                    || signal == WorkflowSignal.LOAD_REQUESTED
                    || signal == WorkflowSignal.PROFILE_APPLIED;
            case DELIVERY -> signal == WorkflowSignal.DELIVERY_REQUESTED
                    || signal == WorkflowSignal.DELIVERY_ACKNOWLEDGED;
            case TRANSFER -> signal == WorkflowSignal.TRANSFER_REQUESTED
                    || signal == WorkflowSignal.SAVE_ACKNOWLEDGED
                    || signal == WorkflowSignal.CONNECT_REQUESTED
                    || signal == WorkflowSignal.CONNECT_COMPLETED
                    || signal == WorkflowSignal.ARRIVAL_ATTESTED;
        };
    }

    private boolean requiresProof(WorkflowSignal signal) {
        return signal == WorkflowSignal.SAVE_ACKNOWLEDGED
                || signal == WorkflowSignal.PROFILE_APPLIED
                || signal == WorkflowSignal.DELIVERY_ACKNOWLEDGED
                || signal == WorkflowSignal.ARRIVAL_ATTESTED;
    }

    private WorkflowDecision advance(WorkflowPhase phase, WorkflowSignal signal, String failure) {
        var history = new ArrayList<>(view.replayHistory());
        history.add(new WorkflowReplay(signal, replayFailure(signal, failure)));
        view = new WorkflowView(view.kind(), view.key(), phase, view.viewRevision() + 1, signal,
                normalize(failure), history);
        return new WorkflowDecision(WorkflowDecision.Outcome.APPLIED, view, "applied");
    }

    private WorkflowDecision denied(String reason) {
        return new WorkflowDecision(WorkflowDecision.Outcome.DENIED, view, reason);
    }

    private String normalize(String value) {
        return value == null ? "" : value;
    }

    private String replayFailure(WorkflowSignal signal, String failure) {
        return signal == WorkflowSignal.FAILED ? normalize(failure) : "";
    }

    private WorkflowDecision duplicate() {
        return new WorkflowDecision(WorkflowDecision.Outcome.DUPLICATE, view, "stable duplicate");
    }
}
