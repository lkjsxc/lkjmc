package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import java.time.Duration;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;

public final class ServerHeartbeat {
    private final SchedulerBridge scheduler;
    private final Optional<DaemonClient> daemon;
    private final String instanceId;

    public ServerHeartbeat(SchedulerBridge scheduler, Optional<DaemonClient> daemon, String instanceId) {
        this.scheduler = scheduler;
        this.daemon = daemon == null ? Optional.empty() : daemon;
        this.instanceId = instanceId;
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
            Map.of("id", instanceId)
        );
        daemon.get().send(request);
    }
}
