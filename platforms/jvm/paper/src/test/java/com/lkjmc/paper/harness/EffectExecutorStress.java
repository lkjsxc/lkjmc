package com.lkjmc.paper.harness;

import com.lkjmc.common.effect.BoundedEffectExecutor;
import com.lkjmc.common.effect.EffectTask;
import java.time.Duration;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;

final class EffectExecutorStress {
    static void queueSaturation() throws Exception {
        var started = new CountDownLatch(1);
        var heldStage = new CompletableFuture<String>();
        var effects = new BoundedEffectExecutor("saturation", 1, 1);
        try {
            var held = effects.submit(task("held", () -> {
                started.countDown();
                return heldStage;
            }));
            check(started.await(2, TimeUnit.SECONDS), "worker did not start");
            var queued = effects.submit(task("queued", CompletableFuture::new));
            long start = System.nanoTime();
            var overloaded = effects.submit(task("overloaded", CompletableFuture::new));
            check(Duration.ofNanos(System.nanoTime() - start).toMillis() < 100,
                    "queue rejection blocked");
            try {
                overloaded.get(2, TimeUnit.SECONDS);
                throw new IllegalStateException("saturated queue accepted");
            } catch (ExecutionException expected) {
                check(expected.getCause() instanceof IllegalStateException,
                        "saturation did not report overload");
            }
            effects.close();
            check(held.isCompletedExceptionally() && queued.isCompletedExceptionally(),
                    "shutdown left effect results incomplete");
            check(effects.awaitClosed(Duration.ofSeconds(2)), "saturated executor did not close");
        } finally {
            effects.close();
        }
    }

    private static EffectTask<String> task(
            String name,
            java.util.function.Supplier<java.util.concurrent.CompletionStage<String>> operation) {
        return new EffectTask<>(name, 1, Duration.ofSeconds(10), operation);
    }

    private static void check(boolean condition, String message) {
        if (!condition) throw new IllegalStateException(message);
    }

    private EffectExecutorStress() {}
}
