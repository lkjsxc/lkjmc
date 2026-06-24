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

    public void initialize() {
        new VelocityCommands(proxy).register();
        logger.info("registered lkjmc Velocity commands");
    }
}
