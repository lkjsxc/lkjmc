package com.lkjmc.velocity;

import com.lkjmc.common.attestation.AttestationVerifier;
import com.lkjmc.common.effect.BoundedEffectExecutor;
import com.lkjmc.common.effect.EffectTask;
import com.lkjmc.common.workflow.WorkflowDecision;
import com.lkjmc.common.workflow.WorkflowKey;
import com.lkjmc.common.workflow.WorkflowMachine;
import com.lkjmc.common.workflow.WorkflowPhase;
import com.lkjmc.common.workflow.WorkflowSignal;
import com.lkjmc.common.workflow.WorkflowView;
import java.time.Duration;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;

public final class VelocityTransferAdapter {
    private final RoutingPlatform platform;
    private final BoundedEffectExecutor effects;
    private final AttestationVerifier verifier;

    public VelocityTransferAdapter(
            RoutingPlatform platform,
            BoundedEffectExecutor effects,
            AttestationVerifier verifier) {
        this.platform = platform;
        this.effects = effects;
        this.verifier = verifier;
    }

    public CompletionStage<WorkflowDecision> connect(
            WorkflowMachine workflow,
            WorkflowKey key,
            String routeId) {
        if (workflow.view().phase() != WorkflowPhase.SAVE_ACKNOWLEDGED) {
            return CompletableFuture.completedFuture(denied(workflow.view(), "save is not acknowledged"));
        }
        WorkflowDecision requested = workflow.apply(key, WorkflowSignal.CONNECT_REQUESTED, false, "");
        if (requested.outcome() == WorkflowDecision.Outcome.DENIED) {
            return CompletableFuture.completedFuture(requested);
        }
        EffectTask<Boolean> task = new EffectTask<>("velocity-connect", 1, Duration.ofSeconds(5),
                () -> platform.connect(key.playerId(), VelocityRoutingAdapter.owned(routeId)));
        return effects.submit(task).handle((connected, failure) -> {
            if (failure != null || !Boolean.TRUE.equals(connected)) {
                return workflow.apply(key, WorkflowSignal.FAILED, false, "connection failed");
            }
            return workflow.apply(key, WorkflowSignal.CONNECT_COMPLETED, false, "");
        });
    }

    public CompletionStage<WorkflowDecision> attestArrival(
            WorkflowMachine workflow,
            WorkflowKey key) {
        if (workflow.view().phase() != WorkflowPhase.CONNECTED) {
            return CompletableFuture.completedFuture(denied(workflow.view(), "connection not completed"));
        }
        EffectTask<AttestationVerifier.Attestation> task = new EffectTask<>(
                "arrival-attestation", 1, Duration.ofSeconds(2), () -> verifier.verify(key));
        return effects.submit(task).handle((attestation, failure) -> {
            boolean trusted = failure == null && attestation.trusted() && key.equals(attestation.key());
            return workflow.apply(key, WorkflowSignal.ARRIVAL_ATTESTED, trusted, "");
        });
    }

    private WorkflowDecision denied(WorkflowView view, String reason) {
        return new WorkflowDecision(WorkflowDecision.Outcome.DENIED, view, reason);
    }
}
