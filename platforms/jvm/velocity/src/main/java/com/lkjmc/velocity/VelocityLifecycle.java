package com.lkjmc.velocity;

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
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.function.Consumer;

public final class VelocityLifecycle implements AutoCloseable {
    private static final Map<String, RoutingTarget> FIXED_BACKEND_ROUTES = Map.of(
            "hub", new RoutingTarget("127.0.0.1", "hub", 25566),
            "survival", new RoutingTarget("127.0.0.1", "survival", 25567));

    private final ProxyServer proxy;
    private final Consumer<String> diagnosticSink;
    private final SerializedRuntimeOwner owner = new SerializedRuntimeOwner(Duration.ofSeconds(2));
    private final List<Object> listeners = new ArrayList<>();
    private Object registeredPlugin;
    private CommandMeta registeredCommandMeta;
    private volatile LkjmcVelocityCommand registeredCommand;

    public VelocityLifecycle(ProxyServer proxy) {
        this(proxy, System.err::println);
    }

    public VelocityLifecycle(ProxyServer proxy, Consumer<String> diagnosticSink) {
        this.proxy = proxy;
        this.diagnosticSink = diagnosticSink;
    }

    public CompletableFuture<Void> initialize(Object plugin) {
        if (plugin == null) throw new IllegalArgumentException("plugin required");
        return owner.replace(this::unregisterSurface, () -> {
            var coordinator = SyncBootstrap.fromEnvironment(System.getenv());
            return new JvmPluginRuntime(coordinator, "velocity", diagnosticSink);
        }, runtime -> install(plugin, runtime));
    }

    private void install(Object plugin, JvmPluginRuntime runtime) {
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

            registeredCommand = new LkjmcVelocityCommand(proxy, diagnosticSink);
            var commandManager = proxy.getCommandManager();
            registeredCommandMeta = commandManager.metaBuilder("lkjmc").plugin(plugin).build();
            commandManager.register(registeredCommandMeta, registeredCommand.command());
            if (!commandManager.hasCommand("lkjmc")) {
                throw new IllegalStateException("/lkjmc command registration was not retained");
            }
            diagnosticSink.accept("lkjmc Velocity command registered: /lkjmc status | "
                    + "/lkjmc server <hub|survival>");
            var platform = new VelocityProxyPlatform(proxy);
            for (String id : LkjmcVelocityCommand.SERVER_IDS) {
                var actual = platform.route(id);
                if (actual.isEmpty()) {
                    throw new IllegalStateException("fixed backend registration is missing: " + id);
                }
                if (!FIXED_BACKEND_ROUTES.get(id).equals(actual.get())) {
                    throw new IllegalStateException("fixed backend registration route is invalid: " + id);
                }
            }
            diagnosticSink.accept("lkjmc fixed backend registrations verified: "
                    + "hub=127.0.0.1:25566,survival=127.0.0.1:25567");

            var scheduler = new VelocitySchedulerBridge(proxy, plugin);
            new VelocityRoutingAdapter(platform, scheduler);
            new VelocityTransferAdapter(platform, runtime.effects(), AttestationVerifier.unavailable());
            runtime.startHeartbeat();
        } catch (RuntimeException failure) {
            unregisterSurface();
            throw failure;
        }
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
