package com.lkjmc.velocity;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.lkjmc.common.attestation.AttestationVerifier;
import com.lkjmc.common.diagnostic.DiagnosticEvent;
import com.lkjmc.common.runtime.JvmPluginRuntime;
import com.lkjmc.common.runtime.SerializedRuntimeOwner;
import com.lkjmc.common.sync.SyncBootstrap;
import com.lkjmc.common.sync.SyncKey;
import com.velocitypowered.api.command.CommandMeta;
import com.velocitypowered.api.proxy.ProxyServer;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Consumer;

public final class VelocityLifecycle implements AutoCloseable {
    private final ProxyServer proxy;
    private final Consumer<String> diagnosticSink;
    private final List<String> backendIds;
    private final SerializedRuntimeOwner owner = new SerializedRuntimeOwner(Duration.ofSeconds(2));
    private final List<Object> listeners = new ArrayList<>();
    private Object registeredPlugin;
    private CommandMeta registeredCommandMeta;
    private volatile LkjmcVelocityCommand registeredCommand;

    public VelocityLifecycle(ProxyServer proxy) {
        this(proxy, System.err::println, configuredBackendIds(System.getenv()));
    }

    public VelocityLifecycle(ProxyServer proxy, Consumer<String> diagnosticSink) {
        this(proxy, diagnosticSink, configuredBackendIds(System.getenv()));
    }

    VelocityLifecycle(
            ProxyServer proxy, Consumer<String> diagnosticSink, List<String> backendIds) {
        this.proxy = proxy;
        this.diagnosticSink = diagnosticSink;
        this.backendIds = List.copyOf(backendIds);
    }

    public CompletableFuture<Void> initialize(Object plugin) {
        if (plugin == null) throw new IllegalArgumentException("plugin required");
        var platform = new VelocityProxyPlatform(proxy);
        var heartbeatRegistrations = new AtomicReference<List<ExpectedRegistration>>(List.of());
        CompletableFuture<Void> initialized = owner.replace(this::unregisterSurface, () -> {
            var coordinator = SyncBootstrap.fromEnvironment(System.getenv());
            return new JvmPluginRuntime(
                    coordinator,
                    "velocity",
                    diagnosticSink,
                    () -> registrationPayload(platform, heartbeatRegistrations.get()));
        }, runtime -> install(plugin, runtime, platform, heartbeatRegistrations));
        initialized.whenComplete((unused, failure) -> {
            if (failure != null) diagnosticSink.accept("lkjmc Velocity initialization failed");
        });
        return initialized;
    }

    private void install(
            Object plugin,
            JvmPluginRuntime runtime,
            RoutingPlatform platform,
            AtomicReference<List<ExpectedRegistration>> heartbeatRegistrations) {
        runtime.diagnostics().emit(DiagnosticEvent.local("velocity",
                DiagnosticEvent.EventKind.RUNTIME_DIAGNOSTIC,
                DiagnosticEvent.Outcome.SUCCEEDED,
                Map.of("serverId", "velocity", "runtime", "presentation")));
        runtime.subscribe(List.of(new SyncKey("routing", "network")));
        registeredPlugin = plugin;
        try {
            listeners.add(new VelocityMotdAdapter());
            listeners.add(new VelocityTabListAdapter(proxy));
            listeners.forEach(listener -> proxy.getEventManager().register(plugin, listener));

            registeredCommand = new LkjmcVelocityCommand(proxy, diagnosticSink, backendIds);
            var commandManager = proxy.getCommandManager();
            registeredCommandMeta = commandManager.metaBuilder("lkjmc").plugin(plugin).build();
            commandManager.register(registeredCommandMeta, registeredCommand.command());
            if (!commandManager.hasCommand("lkjmc")) {
                throw new IllegalStateException("/lkjmc command registration was not retained");
            }
            diagnosticSink.accept("lkjmc Velocity command registered: /lkjmc status | "
                    + "/lkjmc server <instance-id>");
            List<ExpectedRegistration> expected = new ArrayList<>();
            for (String id : backendIds) {
                var actual = platform.route(id);
                if (actual.isEmpty()) {
                    throw new IllegalStateException("configured backend registration is missing: " + id);
                }
                expected.add(new ExpectedRegistration(id, actual.orElseThrow()));
            }
            heartbeatRegistrations.set(List.copyOf(expected));
            diagnosticSink.accept("lkjmc Velocity backend registrations verified: "
                    + String.join(",", backendIds));

            var scheduler = new VelocitySchedulerBridge(proxy, plugin);
            new VelocityRoutingAdapter(platform, scheduler);
            new VelocityTransferAdapter(platform, runtime.effects(), AttestationVerifier.unavailable());
            runtime.startHeartbeat();
        } catch (RuntimeException failure) {
            unregisterSurface();
            throw failure;
        }
    }

    static String registrationPayload(
            RoutingPlatform platform, List<ExpectedRegistration> expected) {
        if (expected == null || expected.isEmpty() || expected.size() > 64) {
            throw new IllegalStateException("bounded expected backend registrations required");
        }
        JsonArray registrations = new JsonArray();
        for (ExpectedRegistration item : expected) {
            var actual = platform.route(item.instanceId());
            boolean registered = actual.filter(item.route()::equals).isPresent();
            JsonObject observation = new JsonObject();
            observation.addProperty("instanceId", item.instanceId());
            observation.addProperty("connectHost", item.route().host());
            observation.addProperty("connectPort", item.route().port());
            observation.addProperty("registered", registered);
            if (!registered) {
                observation.addProperty(
                        "failureReason", actual.isEmpty() ? "missing-registration" : "route-mismatch");
            }
            registrations.add(observation);
        }
        JsonObject payload = new JsonObject();
        payload.add("registrations", registrations);
        return payload.toString();
    }

    record ExpectedRegistration(String instanceId, RoutingTarget route) {}

    private static List<String> configuredBackendIds(Map<String, String> environment) {
        String value = environment.get("LKJMC_BACKEND_IDS");
        if (value == null || value.isBlank()) {
            throw new IllegalStateException("LKJMC_BACKEND_IDS is required");
        }
        return Arrays.asList(value.split(",", -1));
    }

    private void unregisterSurface() {
        if (registeredCommand != null) {
            registeredCommand.close();
        }
        if (registeredCommandMeta != null) {
            proxy.getCommandManager().unregister(registeredCommandMeta);
        }
        registeredCommandMeta = null;
        registeredCommand = null;
        if (registeredPlugin != null) {
            listeners.forEach(listener ->
                    proxy.getEventManager().unregisterListener(registeredPlugin, listener));
        }
        listeners.clear();
        registeredPlugin = null;
    }

    public boolean awaitIdle(Duration timeout) throws InterruptedException {
        return owner.awaitIdle(timeout);
    }

    public int activeRuntimes() {
        return owner.activeRuntimes();
    }

    public int maximumActiveRuntimes() {
        return owner.maximumActiveRuntimes();
    }

    @Override
    public void close() {
        LkjmcVelocityCommand command = registeredCommand;
        if (command != null) command.close();
        owner.closeAsync(this::unregisterSurface);
    }
}
