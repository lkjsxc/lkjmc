package com.lkjmc.common.ui.kernel;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.document.MenuDocumentLoader;
import com.lkjmc.common.ui.document.MenuDocumentSet;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Random;
import org.junit.jupiter.api.Test;

final class UiUpdateBehaviorTest {
    private static final MenuDocumentSet DOCS = MenuDocumentLoader.fromResources();

    @Test
    void openValidatesRoutesAndMaintainsStack() {
        var ids = new SeqIds();
        var root = UiModel.root("initial");
        var travel = UiUpdate.update(DOCS, root, new UiMsg.Open(new MenuRoute("travel")), ids);
        assertEquals(new MenuRoute("travel"), travel.model().route());
        assertEquals(List.of(MenuRoute.root(), new MenuRoute("travel")), travel.model().stack());
        assertInstanceOf(RoutePhase.Static.class, travel.model().phase());
        assertTrue(travel.effects().isEmpty());

        var shop = UiUpdate.update(DOCS, travel.model(), new UiMsg.Open(new MenuRoute("shop")), ids);
        assertInstanceOf(RoutePhase.Loading.class, shop.model().phase());
        assertInstanceOf(UiEffect.LoadData.class, shop.effects().getFirst());
        assertEquals("shop", ((UiEffect.LoadData) shop.effects().getFirst()).plan().binding());

        var missing = UiUpdate.update(DOCS, shop.model(), new UiMsg.Open(new MenuRoute("docs-file")), ids);
        assertInstanceOf(RoutePhase.Diagnostic.class, missing.model().phase());

        var reset = UiUpdate.update(DOCS, missing.model(), new UiMsg.Open(MenuRoute.root()), ids);
        assertEquals(List.of(MenuRoute.root()), reset.model().stack());
    }

    @Test
    void openingSameRouteIdReplacesTopInsteadOfPushing() {
        var first = new MenuRoute("docs-file", Map.of("path", "a.md", "page", "0"));
        var second = new MenuRoute("docs-file", Map.of("path", "a.md", "page", "1"));
        var model = new UiModel(first, List.of(MenuRoute.root(), first), "s", 3, new RoutePhase.Loading(), 0);

        var step = UiUpdate.update(DOCS, model, new UiMsg.Open(second), new SeqIds());

        assertEquals(List.of(MenuRoute.root(), second), step.model().stack());
        assertEquals(2, step.model().stack().size());
        assertEquals(0, step.model().page());
    }

    @Test
    void dataTransitionsBumpEpochAndClampLoadedPage() {
        var model = model(new MenuRoute("shop"), new RoutePhase.Loading(), 99);
        var loaded = UiUpdate.update(DOCS, model, new UiMsg.DataLoaded(listView(2)), new SeqIds());
        assertEquals(0, loaded.model().page());
        assertInstanceOf(RoutePhase.Loaded.class, loaded.model().phase());

        assertInstanceOf(RoutePhase.Empty.class,
            UiUpdate.update(DOCS, loaded.model(), new UiMsg.DataEmpty(), new SeqIds()).model().phase());
        assertInstanceOf(RoutePhase.Denied.class,
            UiUpdate.update(DOCS, loaded.model(), new UiMsg.DataDenied(), new SeqIds()).model().phase());
        assertInstanceOf(RoutePhase.Diagnostic.class,
            UiUpdate.update(DOCS, loaded.model(), new UiMsg.DataFailed("daemon.http_failed"), new SeqIds()).model().phase());
        assertInstanceOf(RoutePhase.Stale.class,
            UiUpdate.update(DOCS, loaded.model(), new UiMsg.StaleAvailable(listView(1), "daemon.http_timeout"),
                new SeqIds()).model().phase());
        assertEquals(model.epoch() + 1, loaded.model().epoch());
    }

    @Test
    void backRefreshAndTextUseLoadEffectsWithoutClosing() {
        var shop = new MenuRoute("shop");
        var travel = new MenuRoute("travel");
        var model = new UiModel(travel, List.of(MenuRoute.root(), shop, travel), "s", 5,
            new RoutePhase.Static(), 2);
        var back = UiUpdate.update(DOCS, model, new UiMsg.BackRequested(), new SeqIds());
        assertEquals(shop, back.model().route());
        assertInstanceOf(UiEffect.LoadData.class, back.effects().getFirst());

        var loaded = model(shop, new RoutePhase.Loaded(listView(30)), 1);
        var refresh = UiUpdate.update(DOCS, loaded, new UiMsg.RefreshRequested(), new SeqIds());
        assertEquals(1, refresh.model().page());
        assertInstanceOf(RoutePhase.Loading.class, refresh.model().phase());

        var text = UiUpdate.update(DOCS, loaded, new UiMsg.TextSubmitted("home", "sethome "), new SeqIds());
        assertEquals("sethome home", ((UiEffect.RunCommand) text.effects().getFirst()).command());
        assertInstanceOf(UiEffect.LoadData.class, text.effects().get(1));
    }

    @Test
    void clickedValidatesMetadataAndDispatchesActions() {
        var root = UiModel.root("session");
        var open = click(root, 20, metadata(root, 20), false);
        assertEquals(new MenuRoute("travel"), open.model().route());

        var bad = withSession(metadata(root, 20), "other");
        var stale = click(root, 20, bad, false);
        assertMessage(stale, MenuFailureCode.STALE_SESSION.messageKey());

        var malformed = click(root, 20, metadata(root, 20), true);
        assertMessage(malformed, MenuFailureCode.UNKNOWN_METADATA.messageKey());
        assertTrue(click(root, 4, null, true).effects().isEmpty());
        assertTrue(click(root, 11, null, true).effects().isEmpty());

        var close = click(root, 53, metadata(root, 53), false);
        assertInstanceOf(UiEffect.CloseInventory.class, close.effects().getFirst());
    }

    @Test
    void clickedDispatchesDaemonPromptAndPagination() {
        var language = model(new MenuRoute("language"), new RoutePhase.Static(), 0);
        var daemon = click(language, 20, metadata(language, 20), false);
        assertInstanceOf(UiEffect.SendDaemon.class, daemon.effects().getFirst());
        assertTrue(((UiEffect.SendDaemon) daemon.effects().getFirst()).refreshOnOk());

        var promptModel = model(new MenuRoute("home-create-name"), new RoutePhase.Static(), 0);
        var prompt = click(promptModel, 22, metadata(promptModel, 22), false);
        assertEquals("sethome ", ((UiEffect.PromptText) prompt.effects().getFirst()).commandPrefix());

        var shop = model(new MenuRoute("shop"), new RoutePhase.Loaded(listView(30)), 0);
        var next = click(shop, 48, metadata(shop, 48), false);
        assertEquals(1, next.model().page());
        assertTrue(next.effects().isEmpty());
    }

    @Test
    void stackInvariantsHoldAcrossSeededMessageSweep() {
        var random = new Random(42);
        var ids = new SeqIds();
        var model = UiModel.root("s0");
        var routes = List.of(new MenuRoute("root"), new MenuRoute("travel"), new MenuRoute("shop"));
        for (int i = 0; i < 100; i++) {
            var msg = switch (random.nextInt(4)) {
                case 0 -> new UiMsg.Open(routes.get(random.nextInt(routes.size())));
                case 1 -> new UiMsg.BackRequested();
                case 2 -> new UiMsg.RefreshRequested();
                default -> new UiMsg.DataLoaded(listView(random.nextInt(40)));
            };
            model = UiUpdate.update(DOCS, model, msg, ids).model();
            assertEquals(MenuRoute.root(), model.stack().getFirst());
            assertEquals(model.route(), model.stack().getLast());
            assertTrue(model.page() >= 0);
        }
    }

    private static UiStep click(UiModel model, int slot, MenuMetadata metadata, boolean malformed) {
        return UiUpdate.update(DOCS, model, new UiMsg.Clicked(slot, metadata, malformed), new SeqIds());
    }

    private static MenuMetadata metadata(UiModel model, int slot) {
        return UiView.frame(DOCS, model).slots().stream().filter(value -> value.slot() == slot)
            .findFirst().orElseThrow().metadata();
    }

    private static UiModel model(MenuRoute route, RoutePhase phase, int page) {
        return new UiModel(route, List.of(MenuRoute.root(), route), "session", 7, phase, page);
    }

    private static RouteView.ListView listView(int size) {
        var entries = new ArrayList<EntryView>();
        for (int i = 0; i < size; i++) {
            entries.add(new EntryView("STONE", TextRef.key("menu.root.title"), List.of(), ItemRole.ACTION,
                new DocumentAction.Message("menu.root.title")));
        }
        return new RouteView.ListView(entries, List.of(TextRef.key("menu.root.info")));
    }

    private static void assertMessage(UiStep step, String key) {
        var message = (UiEffect.Message) step.effects().getFirst();
        assertEquals(key, ((TextRef.Key) message.text()).key());
    }

    private static MenuMetadata withSession(MenuMetadata metadata, String session) {
        return new MenuMetadata(metadata.route(), metadata.params(), metadata.slot(), metadata.actionKey(),
            metadata.payload(), session, metadata.epoch());
    }

    private static final class SeqIds implements UiIds {
        private int next = 1;
        @Override public String nextSessionId() { return "s" + next++; }
    }
}
