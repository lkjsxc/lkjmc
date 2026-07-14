package com.lkjmc.paper.harness;

import com.lkjmc.velocity.RoutingTarget;
import com.lkjmc.common.scheduler.PaperScheduler;
import com.lkjmc.common.scheduler.VelocityScheduler;
import com.lkjmc.velocity.RoutingPlatform;
import java.lang.reflect.Proxy;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.atomic.AtomicInteger;
import org.bukkit.entity.Player;
import org.bukkit.inventory.PlayerInventory;

final class HarnessFakes {
    static final class PaperHops implements PaperScheduler {
        final AtomicInteger global = new AtomicInteger();
        final AtomicInteger entity = new AtomicInteger();
        final AtomicInteger region = new AtomicInteger();
        final AtomicInteger async = new AtomicInteger();
        volatile boolean hold;
        volatile Runnable held;

        @Override public CompletionStage<Void> mainOrGlobal(Runnable action) {
            global.incrementAndGet(); return run(action);
        }
        @Override public CompletionStage<Void> entity(UUID ignored, Runnable action) {
            entity.incrementAndGet(); return run(action);
        }
        @Override public CompletionStage<Void> region(String world, int x, int z, Runnable action) {
            region.incrementAndGet(); return run(action);
        }
        @Override public CompletionStage<Void> async(Runnable action) {
            async.incrementAndGet(); return run(action);
        }
        void release() { if (held != null) { hold = false; held.run(); held = null; } }
        private CompletionStage<Void> run(Runnable action) {
            CompletableFuture<Void> result = new CompletableFuture<>();
            Runnable wrapped = () -> {
                try { action.run(); result.complete(null); }
                catch (RuntimeException failure) { result.completeExceptionally(failure); }
            };
            if (hold) held = wrapped; else wrapped.run();
            return result;
        }
    }

    static final class VelocityHops implements VelocityScheduler {
        final AtomicInteger events = new AtomicInteger();
        final AtomicInteger async = new AtomicInteger();
        @Override public CompletionStage<Void> event(Runnable action) {
            events.incrementAndGet(); action.run(); return CompletableFuture.completedFuture(null);
        }
        @Override public CompletionStage<Void> async(Runnable action) {
            async.incrementAndGet(); action.run(); return CompletableFuture.completedFuture(null);
        }
    }

    static final class ProxyEffects implements RoutingPlatform {
        final Set<String> names = new HashSet<>();
        final HashMap<String, RoutingTarget> routes = new HashMap<>();
        boolean connectionResult = true;
        @Override public Set<String> registrations() { return Set.copyOf(names); }
        @Override public Optional<RoutingTarget> route(String id) { return Optional.ofNullable(routes.get(id)); }
        @Override public boolean register(String id, RoutingTarget route) { routes.put(id, route); return names.add(id); }
        @Override public boolean unregister(String id) {
            names.remove(id); routes.remove(id); return !names.contains(id);
        }
        @Override public CompletionStage<Boolean> connect(UUID player, String id) {
            return CompletableFuture.completedFuture(connectionResult && names.contains(id));
        }
    }

    record PlayerFake(Player player, AtomicInteger clears) {}

    static PlayerFake player(UUID id) {
        AtomicInteger clears = new AtomicInteger();
        PlayerInventory inventory = (PlayerInventory) Proxy.newProxyInstance(
                HarnessFakes.class.getClassLoader(), new Class<?>[]{PlayerInventory.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("clear")) { clears.incrementAndGet(); return null; }
                    if (method.getName().equals("getSize")) return 41;
                    return defaultValue(method.getReturnType());
                });
        Player player = (Player) Proxy.newProxyInstance(
                HarnessFakes.class.getClassLoader(), new Class<?>[]{Player.class},
                (proxy, method, args) -> switch (method.getName()) {
                    case "getUniqueId" -> id;
                    case "getInventory" -> inventory;
                    case "updateInventory" -> null;
                    default -> defaultValue(method.getReturnType());
                });
        return new PlayerFake(player, clears);
    }

    private static Object defaultValue(Class<?> type) {
        if (!type.isPrimitive()) return null;
        if (type == boolean.class) return false;
        if (type == char.class) return '\0';
        if (type == byte.class) return (byte) 0;
        if (type == short.class) return (short) 0;
        if (type == int.class) return 0;
        if (type == long.class) return 0L;
        if (type == float.class) return 0F;
        return 0D;
    }

    private HarnessFakes() {}
}
