package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;

import io.papermc.paper.threadedregions.scheduler.ScheduledTask;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;

final class SchedulerTaskRegistryTest {
    @Test
    void completeRemovesOneShotTask() {
        SchedulerTaskRegistry registry = new SchedulerTaskRegistry();
        AtomicInteger cancels = new AtomicInteger();
        ScheduledTask task = task(cancels);
        registry.track(task);
        registry.complete(task);
        registry.cancelAll();
        assertEquals(0, cancels.get());
        assertEquals(0, registry.size());
    }

    @Test
    void cancelAllIsSafeWhileTasksAreTrackedConcurrently() throws Exception {
        SchedulerTaskRegistry registry = new SchedulerTaskRegistry();
        AtomicInteger cancels = new AtomicInteger();
        CountDownLatch start = new CountDownLatch(1);
        List<Thread> threads = new ArrayList<>();
        for (int index = 0; index < 8; index++) {
            Thread thread = new Thread(() -> trackMany(registry, cancels, start));
            threads.add(thread);
            thread.start();
        }
        start.countDown();
        registry.cancelAll();
        for (Thread thread : threads) {
            thread.join();
        }
        registry.cancelAll();
        assertEquals(0, registry.size());
    }

    private static void trackMany(
        SchedulerTaskRegistry registry,
        AtomicInteger cancels,
        CountDownLatch start
    ) {
        try {
            start.await();
            for (int count = 0; count < 100; count++) {
                registry.track(task(cancels));
            }
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
        }
    }

    private static ScheduledTask task(AtomicInteger cancels) {
        return (ScheduledTask) Proxy.newProxyInstance(
            ScheduledTask.class.getClassLoader(),
            new Class<?>[] {ScheduledTask.class},
            (proxy, method, args) -> {
                if ("cancel".equals(method.getName())) {
                    cancels.incrementAndGet();
                    return null;
                }
                return switch (method.getReturnType().getName()) {
                    case "boolean" -> false;
                    case "int" -> 0;
                    case "long" -> 0L;
                    default -> null;
                };
            }
        );
    }
}
