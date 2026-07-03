package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonAccess;
import com.lkjmc.common.permission.PermissionSnapshotCache;
import com.velocitypowered.api.proxy.ProxyServer;
import java.time.Duration;
import org.slf4j.Logger;

public final class VelocityLifecycle {
    private final ProxyServer proxy;
    private final Logger logger;

    public VelocityLifecycle(ProxyServer proxy, Logger logger) {
        this.proxy = proxy;
        this.logger = logger;
    }

    public void initialize(Object plugin) {
        var access = DaemonAccess.fromEnv();
        if (!access.available()) {
            logger.warn("lkjmc daemon access unavailable: {}", access.code());
        }
        var daemon = access.client();
        var registry = daemon.map(client -> new VelocityServerRegistry(proxy, client));
        var adminGrants = daemon.map(client -> new PermissionSnapshotCache(client,
            "velocity-plugin", "velocity")).orElseGet(PermissionSnapshotCache::disabled);
        var transfers = new VelocityProfileTransferBridge();
        transfers.register(proxy, plugin);
        new VelocityCommands(proxy, daemon, registry,
            new VelocityRestartAdapter(proxy, plugin), transfers, adminGrants).register();
        proxy.getEventManager().register(plugin, new VelocityMotdAdapter());
        proxy.getEventManager().register(plugin, new VelocityTabListAdapter(proxy));
        daemon.ifPresent(client -> proxy.getEventManager().register(plugin, new VelocityModerationListener(client)));
        if (adminGrants.enabled()) {
            new VelocityAdminGrantRefresh(proxy, adminGrants).register(plugin);
        }
        registry.ifPresent(value -> {
            value.refresh();
            proxy.getScheduler()
                .buildTask(plugin, value::refresh)
                .repeat(Duration.ofSeconds(10))
                .schedule();
        });
        logger.info("registered lkjmc Velocity commands and listeners");
    }
}
