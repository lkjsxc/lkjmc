package com.lkjmc.velocity;

import static org.junit.jupiter.api.Assertions.*;

import com.velocitypowered.api.event.EventManager;
import com.velocitypowered.api.proxy.ProxyServer;
import java.lang.reflect.Proxy;
import java.time.Duration;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;

final class VelocityLifecycleTest {
    @Test
    void oneHundredCyclesLeaveNoListenerRuntimeOrOverlappingThread() throws Exception {
        Set<Object> listeners = Collections.newSetFromMap(new IdentityHashMap<>());
        AtomicInteger maximumListeners = new AtomicInteger();
        EventManager events = (EventManager) Proxy.newProxyInstance(getClass().getClassLoader(),
                new Class<?>[] {EventManager.class}, (target, method, arguments) -> {
                    if (method.getName().equals("register") && arguments.length == 2) {
                        listeners.add(arguments[1]);
                        maximumListeners.accumulateAndGet(listeners.size(), Math::max);
                        return null;
                    }
                    if (method.getName().equals("unregisterListener")) {
                        listeners.remove(arguments[1]);
                        return null;
                    }
                    return defaultValue(method.getReturnType());
                });
        ProxyServer proxy = (ProxyServer) Proxy.newProxyInstance(getClass().getClassLoader(),
                new Class<?>[] {ProxyServer.class}, (target, method, arguments) ->
                        method.getName().equals("getEventManager") ? events
                                : defaultValue(method.getReturnType()));
        VelocityLifecycle lifecycle = new VelocityLifecycle(proxy);
        Object plugin = new Object();
        for (int cycle = 0; cycle < 100; cycle++) lifecycle.initialize(plugin);
        lifecycle.close();
        assertTrue(lifecycle.awaitIdle(Duration.ofSeconds(10)));
        assertEquals(0, listeners.size());
        assertEquals(2, maximumListeners.get());
        assertEquals(0, lifecycle.activeRuntimes());
        assertEquals(1, lifecycle.maximumActiveRuntimes());
        assertTrue(Thread.getAllStackTraces().keySet().stream().noneMatch(thread -> thread.isAlive()
                && (thread.getName().startsWith("lkjmc-effect-velocity")
                    || thread.getName().equals("lkjmc-runtime-lifecycle"))));
    }

    private static Object defaultValue(Class<?> type) {
        if (!type.isPrimitive()) return null;
        if (type == boolean.class) return false;
        if (type == char.class) return '\0';
        return 0;
    }
}
