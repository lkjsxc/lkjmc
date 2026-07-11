package com.lkjmc.common.ui.kernel;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.ui.document.MenuDocumentLoader;
import com.lkjmc.common.ui.document.MenuDocumentSet;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

final class UiSafetyContainmentTest {
    private static final MenuDocumentSet DOCS = MenuDocumentLoader.fromResources();

    @Test
    void menuOldResponseDropped() {
        var shop = UiUpdate.update(DOCS, UiModel.root("session"), new UiMsg.Open(new MenuRoute("shop")));
        var request = ((UiEffect.LoadData) shop.effects().getFirst()).request();
        var newer = UiUpdate.update(DOCS, shop.model(), new UiMsg.Open(new MenuRoute("travel")));

        var dropped = UiUpdate.update(DOCS, newer.model(), new UiMsg.DataEmpty(request));

        assertEquals(newer.model(), dropped.model());
        assertTrue(dropped.effects().isEmpty());
    }

    @Test
    void sameEpochForgedRequestAndActionDropped() {
        var opened = UiUpdate.update(DOCS, UiModel.root("session"), new UiMsg.Open(new MenuRoute("shop")));
        var request = ((UiEffect.LoadData) opened.effects().getFirst()).request();
        var wrongId = new UiRequest("", request.sessionId(), request.route(), request.epoch(),
            "forged", request.actionKey());
        var wrongAction = new UiRequest("", request.sessionId(), request.route(), request.epoch(),
            request.requestId(), "forged");

        assertDropped(opened.model(), wrongId);
        assertDropped(opened.model(), wrongAction);
    }

    @Test
    void adventureConfirmationCarriesExplicitEulaAcceptance() {
        var route = new MenuRoute("adventures-end-confirm");
        var model = new UiModel(route, List.of(MenuRoute.root(), route), "session", 5,
            new RoutePhase.Static(), 0);
        var frame = UiView.frame(DOCS, model);
        assertEquals("menu.adventures.end.eula.title", ((TextRef.Key) frame.title()).key());
        var slot = frame.slots().stream().filter(value -> value.slot() == 11).findFirst().orElseThrow();
        assertEquals("menu.adventures.end.eula.accept", ((TextRef.Key) slot.name()).key());
        assertEquals("menu.adventures.end.eula.accept.lore", ((TextRef.Key) slot.lore().getFirst()).key());

        var step = UiUpdate.update(DOCS, model, new UiMsg.Clicked(11, slot.metadata(), false));
        var effect = assertInstanceOf(UiEffect.SendDaemon.class, step.effects().getFirst());
        assertEquals("adventure.purchase", effect.plan().command());
        assertEquals("true", effect.plan().body().get("acceptMinecraftEula"));
    }

    @Test
    void ordinaryConfirmationOmitsEulaAcceptance() {
        var route = new MenuRoute("claim-confirm", Map.of("claimId", "claim"));
        var model = new UiModel(route, List.of(MenuRoute.root(), route), "session", 5,
            new RoutePhase.Static(), 0);
        var slot = UiView.frame(DOCS, model).slots().stream().filter(value -> value.slot() == 11)
            .findFirst().orElseThrow();

        var step = UiUpdate.update(DOCS, model, new UiMsg.Clicked(11, slot.metadata(), false));
        var effect = assertInstanceOf(UiEffect.SendDaemon.class, step.effects().getFirst());

        assertFalse(effect.plan().body().containsKey("acceptMinecraftEula"));
    }

    @Test
    void staleActionsDisabled() {
        var route = new MenuRoute("adventures-end-confirm");
        var model = new UiModel(route, List.of(MenuRoute.root(), route), "session", 5,
            new RoutePhase.Stale(new RouteView.DetailView(List.of(), List.of()), "daemon.http_timeout"), 0);

        var action = UiView.frame(DOCS, model).slots().stream().filter(slot -> slot.slot() == 11).findFirst().orElseThrow();

        assertEquals("disabled", action.metadata().payload().get("type"));
    }

    private static void assertDropped(UiModel model, UiRequest request) {
        var dropped = UiUpdate.update(DOCS, model, new UiMsg.DataEmpty(request));
        assertEquals(model, dropped.model());
        assertTrue(dropped.effects().isEmpty());
    }

    @Test
    void duplicateClickContained() {
        var route = new MenuRoute("language");
        var model = new UiModel(route, List.of(MenuRoute.root(), route), "session", 5, new RoutePhase.Static(), 0);
        var metadata = UiView.frame(DOCS, model).slots().stream().filter(slot -> slot.slot() == 20)
            .findFirst().orElseThrow().metadata();

        var first = UiUpdate.update(DOCS, model, new UiMsg.Clicked(20, metadata, false));
        var duplicate = UiUpdate.update(DOCS, first.model(), new UiMsg.Clicked(20, metadata, false));

        assertInstanceOf(UiEffect.SendDaemon.class, first.effects().getFirst());
        assertEquals(1, first.model().pendingActions().size());
        assertInstanceOf(UiEffect.Message.class, duplicate.effects().getFirst());
    }
}
