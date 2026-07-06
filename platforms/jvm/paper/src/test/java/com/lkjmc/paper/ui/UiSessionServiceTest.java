package com.lkjmc.paper.ui;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.permission.PermissionSnapshotCache;
import com.lkjmc.common.ui.kernel.UiModel;
import com.lkjmc.common.ui.kernel.UiMsg;
import java.util.ArrayList;
import java.util.List;
import org.bukkit.entity.Player;
import org.junit.jupiter.api.Test;

final class UiSessionServiceTest {
    @Test
    void closeClearsMatchingSession() {
        var player = UiTestFixtures.player();
        var renderer = new RecordingRenderer();
        var service = service(player, renderer, (p, e, m, s) -> {});

        service.dispatch(player, new UiMsg.Open(com.lkjmc.common.ui.kernel.MenuRoute.root()));
        var session = service.model(player).orElseThrow().sessionId();

        service.close(player, session);

        assertTrue(service.model(player).isEmpty());
        assertEquals(1, renderer.models.size());
    }

    @Test
    void quitClearsSession() {
        var player = UiTestFixtures.player();
        var service = service(player, new RecordingRenderer(), (p, e, m, s) -> {});

        service.openRoot(player);
        service.quit(player);

        assertTrue(service.model(player).isEmpty());
    }

    static UiSessionService service(Player player, UiSessionService.Renderer renderer,
                                    UiSessionService.Effects effects) {
        return new UiSessionService(UiTestFixtures.docs(), renderer, effects, ignored -> "en",
            PermissionSnapshotCache.disabled(), () -> null, () -> List.of(player));
    }

    static final class RecordingRenderer implements UiSessionService.Renderer {
        final List<UiModel> models = new ArrayList<>();
        @Override public void render(Player player, String locale, UiModel model) {
            models.add(model);
        }
    }
}
