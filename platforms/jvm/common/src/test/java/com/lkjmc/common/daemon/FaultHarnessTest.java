package com.lkjmc.common.daemon;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.time.Duration;
import java.util.EnumSet;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Supplier;
import org.junit.jupiter.api.Test;

final class FaultHarnessTest {
    @Test
    void acknowledgementIsControllableBeforeJvmEffect() {
        var boundary = new ControllableDaemonBoundary();
        var acknowledged = new AtomicBoolean();
        boundary.arm(FaultPoint.BEFORE_JVM_ACKNOWLEDGEMENT);

        assertThrows(InjectedFault.class, () -> boundary.acknowledge(() -> acknowledged.set(true)));
        assertFalse(acknowledged.get());
    }

    @Test
    void deadlineAndCredentialLookupAreControllableWithoutSleeping() {
        var boundary = new ControllableDaemonBoundary();
        boundary.arm(FaultPoint.HTTP_DEADLINE);
        assertThrows(InjectedFault.class,
            () -> boundary.awaitHttp(Duration.ofMillis(5), Duration.ZERO).join());

        boundary.arm(FaultPoint.CREDENTIAL_LOOKUP);
        assertThrows(InjectedFault.class, () -> boundary.credential(() -> "unused"));
    }

    @Test
    void unarmedBoundariesPreserveCompletionAndLookup() {
        var boundary = new ControllableDaemonBoundary();
        var acknowledged = new AtomicBoolean();
        boundary.acknowledge(() -> acknowledged.set(true));

        assertEquals("token", boundary.credential(() -> "token"));
        assertEquals("ok", boundary.awaitHttp(Duration.ofMillis(5), Duration.ZERO).join());
        assertEquals(3, boundary.hits());
        assertEquals(true, acknowledged.get());
    }

    private enum FaultPoint {
        BEFORE_JVM_ACKNOWLEDGEMENT("fault-harness-before-jvm-acknowledgement"),
        HTTP_DEADLINE("fault-harness-http-deadline"),
        CREDENTIAL_LOOKUP("fault-harness-credential-lookup");

        private final String marker;

        FaultPoint(String marker) {
            this.marker = marker;
        }

        String marker() {
            return marker;
        }
    }

    private static final class ControllableDaemonBoundary {
        private final EnumSet<FaultPoint> armed = EnumSet.noneOf(FaultPoint.class);
        private int hits;

        void arm(FaultPoint point) {
            armed.add(point);
        }

        void acknowledge(Runnable acknowledgement) {
            checkpoint(FaultPoint.BEFORE_JVM_ACKNOWLEDGEMENT);
            acknowledgement.run();
        }

        CompletableFuture<String> awaitHttp(Duration deadline, Duration elapsed) {
            checkpoint(FaultPoint.HTTP_DEADLINE);
            if (elapsed.compareTo(deadline) > 0) {
                return CompletableFuture.failedFuture(new IllegalStateException("deadline exceeded"));
            }
            return CompletableFuture.completedFuture("ok");
        }

        String credential(Supplier<String> lookup) {
            checkpoint(FaultPoint.CREDENTIAL_LOOKUP);
            return lookup.get();
        }

        int hits() {
            return hits;
        }

        private void checkpoint(FaultPoint point) {
            hits++;
            if (armed.remove(point)) {
                throw new InjectedFault(point.marker());
            }
        }
    }

    private static final class InjectedFault extends RuntimeException {
        InjectedFault(String marker) {
            super(marker);
        }
    }
}
