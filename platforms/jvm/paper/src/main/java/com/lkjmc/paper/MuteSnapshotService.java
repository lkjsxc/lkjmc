package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import java.time.Duration;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicReference;
import org.bukkit.entity.Player;

final class MuteSnapshotService {
    private static final Duration REFRESH_INTERVAL = Duration.ofSeconds(30);
    private static final long MAX_AGE_NANOS = REFRESH_INTERVAL.toNanos();

    private final LkjmcPaperPlugin plugin;
    private final Map<UUID, String> trackedPlayers = new ConcurrentHashMap<>();
    private final AtomicReference<Snapshot> snapshot = new AtomicReference<>(Snapshot.empty());

    MuteSnapshotService(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
    }

    void start() {
        plugin.getServer().getOnlinePlayers().forEach(this::refresh);
        plugin.scheduler().runAsyncRepeating(this::refreshTrackedPlayers,
            REFRESH_INTERVAL, REFRESH_INTERVAL);
    }

    void track(Player player) {
        trackedPlayers.put(player.getUniqueId(), player.getName());
    }

    void refresh(Player player) {
        track(player);
        refresh(player.getUniqueId(), player.getName());
    }

    void remove(Player player) {
        trackedPlayers.remove(player.getUniqueId());
        snapshot.updateAndGet(current -> current.without(player.getUniqueId()));
    }

    Optional<MuteStatus> current(UUID playerUuid) {
        return snapshot.get().current(playerUuid, System.nanoTime());
    }

    private void refreshTrackedPlayers() {
        trackedPlayers.forEach(this::refresh);
    }

    private void refresh(UUID playerUuid, String playerName) {
        var daemon = plugin.daemon();
        if (daemon.isEmpty()) {
            return;
        }
        var request = new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId()),
            "player.moderation.status",
            Map.of("playerUuid", playerUuid.toString(), "playerName", playerName)
        );
        daemon.get().send(request).whenComplete((response, error) -> {
            if (error != null || response == null || !response.ok()) {
                return;
            }
            var muted = DaemonJson.bool(response.body(), "muted");
            var reason = DaemonJson.string(response.body(), "muteReason").orElse("");
            var refreshedAt = System.nanoTime();
            snapshot.updateAndGet(current -> current.with(playerUuid, muted, reason, refreshedAt));
        });
    }

    record MuteStatus(String reason) {}

    private record Snapshot(Map<UUID, Entry> entries) {
        static Snapshot empty() {
            return new Snapshot(Map.of());
        }

        Optional<MuteStatus> current(UUID playerUuid, long nowNanos) {
            var entry = entries.get(playerUuid);
            if (entry == null || nowNanos - entry.refreshedAtNanos() > MAX_AGE_NANOS) {
                return Optional.empty();
            }
            return Optional.of(new MuteStatus(entry.reason()));
        }

        Snapshot with(UUID playerUuid, boolean muted, String reason, long refreshedAtNanos) {
            var next = new HashMap<>(entries);
            if (muted) {
                next.put(playerUuid, new Entry(reason, refreshedAtNanos));
            } else {
                next.remove(playerUuid);
            }
            return new Snapshot(Map.copyOf(next));
        }

        Snapshot without(UUID playerUuid) {
            if (!entries.containsKey(playerUuid)) {
                return this;
            }
            var next = new HashMap<>(entries);
            next.remove(playerUuid);
            return new Snapshot(Map.copyOf(next));
        }
    }

    private record Entry(String reason, long refreshedAtNanos) {}

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
