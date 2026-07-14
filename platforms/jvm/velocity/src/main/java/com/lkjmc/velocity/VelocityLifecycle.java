package com.lkjmc.velocity;

import com.lkjmc.common.attestation.AttestationVerifier;
import com.lkjmc.common.runtime.JvmPluginRuntime;
import com.lkjmc.common.sync.SyncBootstrap;
import com.lkjmc.common.sync.SyncKey;
import com.velocitypowered.api.proxy.ProxyServer;
import java.time.Duration;
import java.util.List;

public final class VelocityLifecycle implements AutoCloseable {
    private final ProxyServer proxy;
    private JvmPluginRuntime runtime;

    public VelocityLifecycle(ProxyServer proxy) {
        this.proxy = proxy;
    }

    public void initialize(Object plugin) {
        if (runtime != null) runtime.closeAsync(Duration.ofSeconds(2));
        proxy.getEventManager().register(plugin, new VelocityMotdAdapter());
        proxy.getEventManager().register(plugin, new VelocityTabListAdapter(proxy));
        runtime = new JvmPluginRuntime(SyncBootstrap.fromEnvironment(System.getenv()), "velocity");
        runtime.subscribe(List.of(new SyncKey("routing", "network")));
        var platform = new VelocityProxyPlatform(proxy);
        var scheduler = new VelocitySchedulerBridge(proxy, plugin);
        new VelocityRoutingAdapter(platform, scheduler);
        new VelocityTransferAdapter(platform, runtime.effects(), AttestationVerifier.unavailable());
    }

    @Override
    public void close() {
        if (runtime != null) {
            runtime.closeAsync(Duration.ofSeconds(2));
            runtime = null;
        }
    }
}
