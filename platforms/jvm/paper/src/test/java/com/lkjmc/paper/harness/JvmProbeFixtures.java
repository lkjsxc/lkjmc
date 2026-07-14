package com.lkjmc.paper.harness;

import com.lkjmc.bindings.Experience;
import com.lkjmc.bindings.ProfileAvailable;
import com.lkjmc.bindings.ProfileEnvelope;
import com.lkjmc.bindings.ProfileSettings;
import com.lkjmc.bindings.ProfileSnapshot;
import com.lkjmc.bindings.RoutingInstance;
import com.lkjmc.bindings.RoutingPayload;
import com.lkjmc.bindings.RoutingPort;
import com.lkjmc.bindings.RoutingSnapshot;
import com.lkjmc.bindings.Vitals;
import com.lkjmc.common.attestation.AttestationVerifier;
import com.lkjmc.common.workflow.WorkflowKey;
import java.time.Instant;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;

final class JvmProbeFixtures {
    private JvmProbeFixtures() {}

    static WorkflowKey key() {
        return new WorkflowKey(UUID.randomUUID(), UUID.randomUUID(), UUID.randomUUID(), 7, 9,
                UUID.randomUUID());
    }

    static ProfileSnapshot profile(WorkflowKey key) {
        var envelope = new ProfileEnvelope("lkjmc-profile-one", List.of(), List.of(), null, 0,
                List.of(), new Experience(0, 0, 0), new Vitals(20, 20, 5, 300), List.of(),
                null, List.of(), List.of(), List.of(), 0, List.of(),
                new ProfileSettings(true, true, true, "default"), "en");
        var payload = new ProfileAvailable(key.playerId(), "global", key.profileRevision(),
                "lkjmc-profile-one", "0".repeat(64), envelope);
        return new ProfileSnapshot("profiles", key.playerId() + ":global", key.profileRevision(),
                Instant.now(), 1, payload);
    }

    static RoutingSnapshot routeSnapshot(Instant now, long revision, String id, int port) {
        var instance = new RoutingInstance(id, "paper", "running", "process-healthy",
                true, true, 0, List.of(new RoutingPort(port, "minecraft")));
        return new RoutingSnapshot("routing", "network", revision, now.minusMillis(1), 1,
                new RoutingPayload(List.of(instance)));
    }

    static AttestationVerifier trusted() {
        return key -> CompletableFuture.completedFuture(new AttestationVerifier.Attestation(key, true));
    }
}
