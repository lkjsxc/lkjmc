package com.lkjmc.velocity;

import com.google.inject.Inject;
import com.lkjmc.common.LkjmcBuildInfo;
import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.proxy.ProxyInitializeEvent;
import com.velocitypowered.api.event.proxy.ProxyShutdownEvent;
import com.velocitypowered.api.plugin.Plugin;
import com.velocitypowered.api.proxy.ProxyServer;
import org.slf4j.Logger;

@Plugin(
    id = "lkjmc",
    name = "lkjmc",
    version = LkjmcBuildInfo.VERSION,
    description = "lkjmc local presentation adapter",
    authors = {"lkjmc"}
)
public final class LkjmcVelocityPlugin {
    private final VelocityLifecycle lifecycle;

    @Inject
    public LkjmcVelocityPlugin(ProxyServer proxy, Logger logger) {
        logger.info("lkjmc version={} commit={} dirty={}", LkjmcBuildInfo.VERSION,
                LkjmcBuildInfo.COMMIT, LkjmcBuildInfo.DIRTY);
        this.lifecycle = new VelocityLifecycle(proxy, logger::info);
    }

    @Subscribe
    public void onProxyInitialize(ProxyInitializeEvent event) {
        lifecycle.initialize(this);
    }

    @Subscribe
    public void onProxyShutdown(ProxyShutdownEvent event) {
        lifecycle.close();
    }
}
