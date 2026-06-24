package com.lkjmc.velocity;

import com.google.inject.Inject;
import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.proxy.ProxyInitializeEvent;
import com.velocitypowered.api.plugin.Plugin;
import com.velocitypowered.api.proxy.ProxyServer;
import org.slf4j.Logger;

@Plugin(
    id = "lkjmc-velocity",
    name = "lkjmc Velocity",
    version = "0.0.0",
    description = "lkjmc Velocity adapter",
    authors = {"lkjmc"}
)
public final class LkjmcVelocityPlugin {
    private final VelocityLifecycle lifecycle;
    private final Logger logger;

    @Inject
    public LkjmcVelocityPlugin(ProxyServer proxy, Logger logger) {
        this.logger = logger;
        this.lifecycle = new VelocityLifecycle(proxy, logger);
    }

    @Subscribe
    public void onProxyInitialize(ProxyInitializeEvent event) {
        lifecycle.initialize();
        logger.info("lkjmc Velocity plugin enabled");
    }
}
