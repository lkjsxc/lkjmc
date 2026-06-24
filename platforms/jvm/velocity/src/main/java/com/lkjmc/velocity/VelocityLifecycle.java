package com.lkjmc.velocity;

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
        new VelocityCommands(proxy).register();
        proxy.getEventManager().register(plugin, new VelocityMotdAdapter());
        proxy.getEventManager().register(plugin, new VelocityTabListAdapter(proxy));
        logger.info("registered lkjmc Velocity commands and listeners");
    }
}
