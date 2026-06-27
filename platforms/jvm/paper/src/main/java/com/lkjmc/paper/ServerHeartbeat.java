package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import java.time.Duration;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;

public final class ServerHeartbeat {
    private final LkjmcPaperPlugin plugin;
    private final SchedulerBridge scheduler;
    private final Optional<DaemonClient> daemon;
    private final String instanceId;
    private final String implementation;

    public ServerHeartbeat(LkjmcPaperPlugin plugin, SchedulerBridge scheduler,
                           Optional<DaemonClient> daemon, String instanceId) {
        this.plugin = plugin;
        this.scheduler = scheduler;
        this.daemon = daemon == null ? Optional.empty() : daemon;
        this.instanceId = instanceId;
        this.implementation = implementation();
    }

    public void start() {
        if (daemon.isEmpty() || instanceId == null || instanceId.isBlank()) {
            return;
        }
        scheduler.runAsyncRepeating(this::send, Duration.ofSeconds(5), Duration.ofSeconds(30));
    }

    private void send() {
        var request = new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId),
            "instance.heartbeat",
            Map.of(
                "id", instanceId,
                "playerCount", plugin.getServer().getOnlinePlayers().size(),
                "maxPlayers", plugin.getServer().getMaxPlayers(),
                "ready", true,
                "implementation", implementation
            )
        );
        daemon.get().send(request);
    }

    private static String implementation() {
        var configured = System.getenv("LKJMC_SERVER_IMPLEMENTATION");
        if (configured != null && !configured.isBlank()) {
            return configured.toLowerCase(Locale.ROOT);
        }
        try {
            Class.forName("io.papermc.paper.threadedregions.RegionizedServer");
            return "folia";
        } catch (ClassNotFoundException ignored) {
            return "paper";
        }
    }
}
