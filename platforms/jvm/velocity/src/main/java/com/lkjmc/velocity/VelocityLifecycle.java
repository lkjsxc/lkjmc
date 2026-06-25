package com.lkjmc.velocity;

import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.HttpDaemonClient;
import com.velocitypowered.api.proxy.ProxyServer;
import org.slf4j.Logger;

public final class VelocityLifecycle {
    private final ProxyServer proxy;
    private final Logger logger;

    public VelocityLifecycle(ProxyServer proxy, Logger logger) {
        this.proxy = proxy;
        this.logger = logger;
    }

    public void initialize(Object plugin) {
        var daemon = HttpDaemonClient.fromEnv().map(client -> (DaemonClient) client);
        var registry = daemon.map(client -> new VelocityServerRegistry(proxy, client));
        var transfers = new VelocityProfileTransferBridge();
        transfers.register(proxy, plugin);
        new VelocityCommands(proxy, daemon, registry, new VelocityRestartAdapter(proxy, plugin), transfers).register();
        proxy.getEventManager().register(plugin, new VelocityMotdAdapter());
        proxy.getEventManager().register(plugin, new VelocityTabListAdapter(proxy));
        daemon.ifPresent(client -> proxy.getEventManager().register(plugin, new VelocityModerationListener(client)));
        registry.ifPresent(VelocityServerRegistry::refresh);
        logger.info("registered lkjmc Velocity commands and listeners");
    }
}
