package com.lkjmc.velocity;

import com.velocitypowered.api.proxy.ProxyServer;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityRestartAdapter {
    private final ProxyServer proxy;
    private final Object plugin;

    public VelocityRestartAdapter(ProxyServer proxy, Object plugin) {
        this.proxy = proxy;
        this.plugin = plugin;
    }

    public void scheduleWarning(int seconds) {
        proxy.sendMessage(VelocityMessages.message(
            "velocity.restart.warning",
            NamedTextColor.RED,
            Map.of("seconds", Integer.toString(seconds))
        ));
        proxy.getScheduler()
            .buildTask(plugin, () -> proxy.sendMessage(VelocityMessages.message(
                "velocity.restart.elapsed", NamedTextColor.RED
            )))
            .delay(Math.max(1, seconds), TimeUnit.SECONDS)
            .schedule();
    }

    public String warningMessage(int seconds) {
        return VelocityMessages.render(
            "velocity.restart.warning",
            Map.of("seconds", Integer.toString(seconds))
        );
    }
}
