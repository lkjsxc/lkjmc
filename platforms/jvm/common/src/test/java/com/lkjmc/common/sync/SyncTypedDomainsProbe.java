package com.lkjmc.common.sync;

import com.lkjmc.bindings.ClaimSnapshot;
import com.lkjmc.bindings.MenuSnapshot;
import com.lkjmc.bindings.PermissionSnapshot;
import com.lkjmc.bindings.PresenceSnapshot;
import com.lkjmc.bindings.ProfileSnapshot;
import com.lkjmc.bindings.RoutingSnapshot;
import com.lkjmc.bindings.SettingsSnapshot;
import java.time.Duration;
import java.util.List;
import java.util.UUID;

final class SyncTypedDomainsProbe {
    private SyncTypedDomainsProbe() {}

    static void run(SyncHarness.Environment environment) throws Exception {
        UUID player = environment.database.player();
        List<SyncKey> keys = List.of(new SyncKey("permissions", "service:sync-harness"),
                new SyncKey("claims", "survival"), new SyncKey("menus", "global"),
                new SyncKey("profiles", player + ":profile"), new SyncKey("presence", "hub"),
                new SyncKey("routing", "network"), new SyncKey("settings", player.toString()));
        List<Class<?>> types = List.of(PermissionSnapshot.class, ClaimSnapshot.class,
                MenuSnapshot.class, ProfileSnapshot.class, PresenceSnapshot.class,
                RoutingSnapshot.class, SettingsSnapshot.class);
        try (SyncCoordinator coordinator = new SyncCoordinator(
                SyncHarness.config(environment, 4, Duration.ofSeconds(2)))) {
            keys.forEach(coordinator::subscribe);
            SyncHarness.check(SyncHarness.await(Duration.ofSeconds(5),
                    () -> keys.stream().allMatch(key -> coordinator.view(key).isPresent())),
                    "not every daemon domain decoded");
            for (int index = 0; index < keys.size(); index++) {
                SyncHarness.check(types.get(index).isInstance(
                        coordinator.view(keys.get(index)).orElseThrow().value()),
                        "wrong generated domain type for " + keys.get(index).domain());
            }
        }
    }
}
