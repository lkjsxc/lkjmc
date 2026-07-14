package com.lkjmc.paper.harness;

import static org.junit.jupiter.api.Assertions.assertEquals;

import com.lkjmc.bindings.PermissionPayload;
import com.lkjmc.bindings.PermissionSnapshot;
import com.lkjmc.bindings.SettingsPayload;
import com.lkjmc.bindings.SettingsSnapshot;
import com.lkjmc.common.menu.MenuResult;
import com.lkjmc.common.menu.MenuSnapshotView;
import com.lkjmc.common.menu.MenuTypes;
import com.lkjmc.paper.PaperMenuProtocolAdapter;
import java.time.Instant;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.Test;

final class MenuMutationTest {
    private final MenuHarnessFixtures fixtures = new MenuHarnessFixtures();
    private final UUID player = UUID.fromString("00000000-0000-0000-0000-000000000009");

    @Test
    void capabilityCannotReplaceAttestationOrMutationPort() {
        var adapter = new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer);
        var opened = (MenuResult.Rendered) adapter.open(20, "language", Map.of(), "en", current());
        var action = opened.frame().bySlot().get(20);
        var unattested = (MenuResult.Failed) adapter.click(action.metadata(), action.action(), false);
        assertEquals(MenuTypes.Failure.UNATTESTED, unattested.failure());
        var unsupported = (MenuResult.Failed) adapter.click(action.metadata(), action.action(), true);
        assertEquals(MenuTypes.Failure.UNSUPPORTED_OPERATION, unsupported.failure());
    }

    @Test
    void staleDependencyDisablesMutation() {
        var permission = permission();
        var settings = settings();
        var view = MenuSnapshotView.of(MenuTypes.Freshness.STALE, permission, settings).withLocalDocs();
        var adapter = new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer);
        var opened = (MenuResult.Rendered) adapter.open(21, "language", Map.of(), "en", view);
        var action = opened.frame().bySlot().get(20);
        var denied = (MenuResult.Failed) adapter.click(action.metadata(), action.action(), true);
        assertEquals(MenuTypes.Failure.DEPENDENCY_STALE, denied.failure());
    }

    private MenuSnapshotView current() {
        return MenuSnapshotView.of(MenuTypes.Freshness.CURRENT, permission(), settings()).withLocalDocs();
    }
    private PermissionSnapshot permission() {
        return new PermissionSnapshot("permissions", player.toString(), 1, Instant.EPOCH, 1,
                new PermissionPayload("player", player.toString(), List.of(),
                        List.of("menu.action.language-set-en")));
    }
    private SettingsSnapshot settings() {
        return new SettingsSnapshot("settings", player.toString(), 1, Instant.EPOCH, 1,
                new SettingsPayload(player, "en", true, true, true, Map.of()));
    }
}
