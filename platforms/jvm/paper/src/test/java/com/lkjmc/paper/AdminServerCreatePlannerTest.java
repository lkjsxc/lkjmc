package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.daemon.DaemonResponse;
import java.lang.reflect.Proxy;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;
import org.junit.jupiter.api.Test;

final class AdminServerCreatePlannerTest {
    private static final UUID PLAYER_ID = UUID.fromString("00000000-0000-0000-0000-000000000123");

    @Test
    void jarMissingDiagnosticBecomesActionablePlanText() {
        var daemon = new FakeDaemon(json("""
            {"startable":false,"diagnostic":{
              "code":"jar_asset_missing",
              "message":"No compatible server jar asset is registered for project/kind 'paper'.",
              "suggestedCommand":"lkjmc jar sync --project paper"}}
            """));
        var plan = new AdminServerCreatePlanner(Optional.of(daemon))
            .plan(player(), "hub", "paper", "paper-survival").join();

        assertFalse(plan.startable());
        assertTrue(plan.diagnostics().contains("lkjmc jar sync --project paper"));
        assertEquals("instance.create.plan", daemon.request.command());
        assertEquals(true, daemon.request.body().get("acceptMinecraftEula"));
    }

    private static Player player() {
        return proxy(Player.class, (proxy, method, args) -> switch (method.getName()) {
            case "getUniqueId" -> PLAYER_ID;
            case "getName" -> "Alex";
            default -> fallback(method.getReturnType());
        });
    }

    private static JsonObject json(String value) {
        return JsonParser.parseString(value).getAsJsonObject();
    }

    @SuppressWarnings("unchecked")
    private static <T> T proxy(Class<T> type, java.lang.reflect.InvocationHandler handler) {
        return (T) Proxy.newProxyInstance(type.getClassLoader(), new Class<?>[] {type}, handler);
    }

    private static Object fallback(Class<?> type) {
        if (type.equals(boolean.class)) return false;
        if (type.equals(int.class)) return 0;
        if (type.equals(void.class)) return null;
        return null;
    }

    private static final class FakeDaemon implements DaemonClient {
        private final JsonObject body;
        private DaemonRequest request;

        private FakeDaemon(JsonObject body) {
            this.body = body;
        }

        @Override
        public CompletableFuture<DaemonResponse> send(DaemonRequest request) {
            this.request = request;
            return CompletableFuture.completedFuture(new DaemonResponse(
                request.requestId(), true, body, Optional.empty()));
        }
    }
}
