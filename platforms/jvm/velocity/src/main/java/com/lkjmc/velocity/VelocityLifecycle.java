package com.lkjmc.velocity;

import com.lkjmc.common.sync.SyncBootstrap;
import com.lkjmc.common.sync.SyncCoordinator;
import com.lkjmc.common.sync.SyncKey;
import com.velocitypowered.api.proxy.ProxyServer;
import java.util.Optional;

public final class VelocityLifecycle implements AutoCloseable {
    private final ProxyServer proxy;
    private Optional<SyncCoordinator> coordinator = Optional.empty();

    public VelocityLifecycle(ProxyServer proxy) {
        this.proxy = proxy;
    }

    public void initialize(Object plugin) {
        proxy.getEventManager().register(plugin, new VelocityMotdAdapter());
        proxy.getEventManager().register(plugin, new VelocityTabListAdapter(proxy));
        coordinator = SyncBootstrap.fromEnvironment(System.getenv());
        coordinator.ifPresent(value -> value.subscribe(new SyncKey("routing", "network")));
    }

    @Override
    public void close() {
        coordinator.ifPresent(SyncCoordinator::close);
        coordinator = Optional.empty();
    }
}
