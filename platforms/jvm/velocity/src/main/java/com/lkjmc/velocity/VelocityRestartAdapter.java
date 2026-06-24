package com.lkjmc.velocity;

import com.velocitypowered.api.proxy.ProxyServer;
import java.util.concurrent.TimeUnit;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityRestartAdapter {
    private final ProxyServer proxy;
    private final Object plugin;

    public VelocityRestartAdapter(ProxyServer proxy, Object plugin) {
        this.proxy = proxy;
        this.plugin = plugin;
    }

    public void scheduleWarning(int seconds) {
        proxy.sendMessage(Component.text(warningMessage(seconds), NamedTextColor.RED));
        proxy.getScheduler()
            .buildTask(plugin, () -> proxy.sendMessage(Component.text(
                "Restart warning elapsed; host supervisor must restart the proxy.",
                NamedTextColor.RED
            )))
            .delay(Math.max(1, seconds), TimeUnit.SECONDS)
            .schedule();
    }

    public String warningMessage(int seconds) {
        return "Proxy restart warning: " + seconds + " seconds";
    }
}
