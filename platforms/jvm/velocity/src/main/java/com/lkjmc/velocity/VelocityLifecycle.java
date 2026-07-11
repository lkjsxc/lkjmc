package com.lkjmc.velocity;

import com.velocitypowered.api.proxy.ProxyServer;

public final class VelocityLifecycle {
    private final ProxyServer proxy;

    public VelocityLifecycle(ProxyServer proxy) {
        this.proxy = proxy;
    }

    public void initialize(Object plugin) {
        proxy.getEventManager().register(plugin, new VelocityMotdAdapter());
        proxy.getEventManager().register(plugin, new VelocityTabListAdapter(proxy));
    }
}
