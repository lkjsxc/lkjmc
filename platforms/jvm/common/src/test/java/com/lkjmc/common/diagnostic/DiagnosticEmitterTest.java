package com.lkjmc.common.diagnostic;

import static org.junit.jupiter.api.Assertions.*;
import java.time.Duration;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CopyOnWriteArrayList;
import org.junit.jupiter.api.Test;

final class DiagnosticEmitterTest {
    @Test
    void emitsTypedBoundedEventAndDrains() {
        List<String> output = new CopyOnWriteArrayList<>();
        var emitter = new DiagnosticEmitter("paper", output::add);
        assertTrue(emitter.emit(DiagnosticEvent.local("paper",
                DiagnosticEvent.EventKind.JVM_DIAGNOSTIC,
                DiagnosticEvent.Outcome.SUCCEEDED,
                Map.of("serverId", "paper"))));
        assertTrue(emitter.close(Duration.ofSeconds(1)));
        assertEquals(1, output.size());
        assertTrue(output.getFirst().contains("\"source\":\"jvm-local\""));
        assertTrue(output.getFirst().contains("\"serverId\":\"paper\""));
    }

    @Test
    void rejectsUnboundedAttributes() {
        assertThrows(IllegalArgumentException.class, () -> DiagnosticEvent.local("paper",
                DiagnosticEvent.EventKind.JVM_DIAGNOSTIC,
                DiagnosticEvent.Outcome.FAILED,
                Map.of("token", "not-allowed")));
    }

    @Test
    void rejectsSecretsAndUrlsInAllowedAttributes() {
        for (String value : List.of("Bearer obs-token-canary", "https://secret.example")) {
            assertThrows(IllegalArgumentException.class, () -> DiagnosticEvent.local("paper",
                    DiagnosticEvent.EventKind.JVM_DIAGNOSTIC,
                    DiagnosticEvent.Outcome.FAILED,
                    Map.of("reason", value)));
        }
    }
}
