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
            registeredCommandMeta = proxy.getCommandManager().metaBuilder("lkjmc").plugin(plugin).build();
            proxy.getCommandManager().register(registeredCommandMeta, registeredCommand.command());

            var platform = new VelocityProxyPlatform(proxy);
            var scheduler = new VelocitySchedulerBridge(proxy, plugin);
            new VelocityRoutingAdapter(platform, scheduler);
            new VelocityTransferAdapter(platform, runtime.effects(), AttestationVerifier.unavailable());
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
