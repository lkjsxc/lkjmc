package com.lkjmc.common.attestation;

import com.lkjmc.common.workflow.WorkflowKey;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;

public interface AttestationVerifier {
    CompletionStage<Attestation> verify(WorkflowKey key);

    record Attestation(WorkflowKey key, boolean trusted) {
        public Attestation {
            if (key == null) throw new IllegalArgumentException("workflow key required");
        }
    }

    static AttestationVerifier unavailable() {
        return key -> CompletableFuture.completedFuture(new Attestation(key, false));
    }
}
