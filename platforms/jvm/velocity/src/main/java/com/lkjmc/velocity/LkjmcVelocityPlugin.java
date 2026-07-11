package com.lkjmc.velocity;

import com.google.inject.Inject;
import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.proxy.ProxyInitializeEvent;
import com.velocitypowered.api.plugin.Plugin;
import com.velocitypowered.api.proxy.ProxyServer;
import org.slf4j.Logger;

@Plugin(
    id = "lkjmc",
    name = "lkjmc",
    version = "0.0.0",
    description = "lkjmc local presentation adapter",
    authors = {"lkjmc"}
)
public final class LkjmcVelocityPlugin {
    private final VelocityLifecycle lifecycle;
    private final Logger logger;

    @Inject
    public LkjmcVelocityPlugin(ProxyServer proxy, Logger logger) {
        this.logger = logger;
        this.lifecycle = new VelocityLifecycle(proxy);
    }

    @Subscribe
    public void onProxyInitialize(ProxyInitializeEvent event) {
        lifecycle.initialize(this);
        logger.info("lkjmc Velocity presentation enabled");
    }
}
