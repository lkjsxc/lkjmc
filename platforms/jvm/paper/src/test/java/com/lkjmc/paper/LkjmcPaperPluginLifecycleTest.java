package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.runtime.JvmPluginRuntime;
import java.time.Duration;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;

final class LkjmcPaperPluginLifecycleTest {
    @Test
    void actualPluginLifecycleOwnsOneListenerSetAcrossOneHundredCycles() throws Exception {
        Set<Object> listeners = Collections.newSetFromMap(new IdentityHashMap<>());
        AtomicInteger maximumListeners = new AtomicInteger();
        AtomicInteger schedulerCalls = new AtomicInteger();
        AtomicReference<JvmPluginRuntime> prior = new AtomicReference<>();
        var lifecycle = new LkjmcPaperPlugin.Lifecycle(Duration.ofSeconds(2), action -> {
            schedulerCalls.incrementAndGet();
            action.run();
            return CompletableFuture.completedFuture(null);
        });
        Runnable uninstall = listeners::clear;
        for (int cycle = 0; cycle < 100; cycle++) {
            replace(lifecycle, listeners, maximumListeners, prior, uninstall, cycle, "enable");
            replace(lifecycle, listeners, maximumListeners, prior, uninstall, cycle, "replacement");
            lifecycle.disable(uninstall).get(3, TimeUnit.SECONDS);
            assertEquals(0, listeners.size(), "listeners after disable cycle=" + cycle);
            assertEquals(0, lifecycle.activeRuntimes(), "runtime after disable cycle=" + cycle);
        }
        assertTrue(lifecycle.awaitIdle(Duration.ofSeconds(2)));
        assertEquals(2, maximumListeners.get());
        assertEquals(1, lifecycle.maximumActiveRuntimes());
        assertTrue(schedulerCalls.get() >= 300);
        assertTrue(Thread.getAllStackTraces().keySet().stream().noneMatch(thread -> thread.isAlive()
                && (thread.getName().startsWith("lkjmc-effect-paper-cycle")
                    || thread.getName().equals("lkjmc-runtime-lifecycle"))));
    }

    private static void replace(LkjmcPaperPlugin.Lifecycle lifecycle, Set<Object> listeners,
            AtomicInteger maximum, AtomicReference<JvmPluginRuntime> prior, Runnable uninstall,
            int cycle, String phase) throws Exception {
        lifecycle.enable(uninstall, () -> new JvmPluginRuntime(Optional.empty(), "paper-cycle"), runtime -> {
            JvmPluginRuntime previous = prior.getAndSet(runtime);
            if (previous != null) assertClosed(previous, cycle, phase);
            listeners.add(new Object());
            listeners.add(new Object());
            maximum.accumulateAndGet(listeners.size(), Math::max);
        }).get(3, TimeUnit.SECONDS);
        assertEquals(2, listeners.size(), "listener set cycle=" + cycle + " phase=" + phase);
        assertEquals(1, lifecycle.activeRuntimes(), "runtime cycle=" + cycle + " phase=" + phase);
    }

    private static void assertClosed(JvmPluginRuntime runtime, int cycle, String phase) {
        try {
            assertTrue(runtime.awaitClosed(Duration.ZERO),
                    "prior close not awaited cycle=" + cycle + " phase=" + phase);
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            throw new AssertionError(interrupted);
        }
    }
}
