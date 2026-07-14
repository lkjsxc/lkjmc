package com.lkjmc.paper;

import com.lkjmc.common.menu.MenuSnapshotView;
import com.lkjmc.common.menu.MenuTypes;
import com.lkjmc.common.runtime.JvmPluginRuntime;
import com.lkjmc.common.sync.SyncKey;
import java.time.Duration;
import java.time.Instant;
import java.util.EnumMap;
import java.util.List;
import java.util.UUID;

final class PaperMenuSnapshots {
    private final JvmPluginRuntime runtime;

    PaperMenuSnapshots(JvmPluginRuntime runtime) { this.runtime = runtime; }

    void subscribe(UUID playerId) { runtime.subscribe(keys(playerId)); }

    MenuSnapshotView view(UUID playerId) {
        var values = new EnumMap<MenuTypes.Domain, MenuSnapshotView.Entry>(MenuTypes.Domain.class);
        runtime.coordinator().ifPresent(coordinator -> {
            for (var key : keys(playerId)) coordinator.view(key).ifPresent(value -> {
                var age = Duration.between(value.receivedAt(), Instant.now());
                var freshness = age.compareTo(Duration.ofSeconds(30)) > 0
                        ? MenuTypes.Freshness.STALE : MenuTypes.Freshness.CURRENT;
                values.put(MenuTypes.Domain.valueOf(key.domain().toUpperCase(java.util.Locale.ROOT)),
                        new MenuSnapshotView.Entry(freshness, value.revision(), value.value()));
            });
        });
        return new MenuSnapshotView(values).withLocalDocs();
    }

    private List<SyncKey> keys(UUID playerId) {
        String id = playerId.toString();
        return List.of(new SyncKey("menus", "global"), new SyncKey("permissions", id),
                new SyncKey("claims", id), new SyncKey("settings", id), new SyncKey("profiles", id),
                new SyncKey("routing", "network"), new SyncKey("presence", "global"));
    }
}
