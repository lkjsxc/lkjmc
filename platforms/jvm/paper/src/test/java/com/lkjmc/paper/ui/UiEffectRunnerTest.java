package com.lkjmc.paper.ui;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonError;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.daemon.DaemonResponse;
import com.lkjmc.common.permission.PermissionSnapshotCache;
import com.lkjmc.common.ui.binding.BindingContext;
import com.lkjmc.common.ui.binding.BindingRegistry;
import com.lkjmc.common.ui.binding.BindingResult;
import com.lkjmc.common.ui.binding.MenuBinding;
import com.lkjmc.common.ui.document.MenuDocument;
import com.lkjmc.common.ui.kernel.DaemonRequestPlan;
import com.lkjmc.common.ui.kernel.RoutePhase;
import com.lkjmc.common.ui.kernel.RouteView;
import com.lkjmc.common.ui.kernel.TextRef;
import com.lkjmc.common.ui.kernel.UiEffect;
import com.lkjmc.common.ui.kernel.UiMsg;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.TextComponent;
import org.bukkit.entity.Player;
import org.junit.jupiter.api.Test;

final class UiEffectRunnerTest {
    @Test
    void daemonCompletionHopsBeforePlayerMutationAndRefresh() {
        var fixture = fixture(new BindingRegistry(List.of()));
        fixture.service.openRoot(fixture.player);
        var model = fixture.service.model(fixture.player).orElseThrow();
        var effect = new UiEffect.SendDaemon(DaemonRequestPlan.command("test.ok", Map.of()),
            TextRef.key("ok"), TextRef.key("fail"), true);

        fixture.runner.run(fixture.player, effect, model, fixture.service);
        fixture.daemon.future.complete(new DaemonResponse(fixture.daemon.request.requestId(), true,
            new JsonObject(), Optional.empty()));

        assertEquals(0, UiTestFixtures.state(fixture.player).messages.size());
        assertEquals(1, fixture.renderer.models.size());
        fixture.scheduler.drain();
        assertEquals(1, UiTestFixtures.state(fixture.player).messages.size());
        assertTrue(fixture.renderer.models.size() > 1);
    }

    @Test
    void typedDiagnosticBeatsGenericFailureKey() {
        var fixture = fixture(new BindingRegistry(List.of()));
        var model = com.lkjmc.common.ui.kernel.UiModel.root("s1");
        var response = new DaemonResponse(java.util.UUID.randomUUID(), false, new JsonObject(),
            Optional.of(new DaemonError("daemon.command_failed", "no", false)));
        fixture.daemon.future.complete(response);

        fixture.runner.run(fixture.player, new UiEffect.SendDaemon(DaemonRequestPlan.command("x", Map.of()),
            TextRef.key("ok"), TextRef.key("fail"), false), model, fixture.service);
        fixture.scheduler.drain();

        var messages = UiTestFixtures.state(fixture.player).messages;
        assertEquals("Typed failed", plain((Component) messages.get(0)));
        assertEquals("Typed hint", plain((Component) messages.get(1)));
    }

    @Test
    void localLoadDataResolvesWithoutRenderingLoadingFrame() {
        var local = UiTestFixtures.document("local", "title.local",
            new MenuDocument.Data("local", MenuDocument.Source.LOCAL, List.of()), List.of());
        var documents = UiTestFixtures.docs(local);
        var renderer = new UiSessionServiceTest.RecordingRenderer();
        var runner = runner(new UiTestFixtures.Scheduler(), new FakeDaemon(),
            new BindingRegistry(List.of(new LocalBinding())), new ArrayList<>());
        var player = UiTestFixtures.player();
        var service = new UiSessionService(documents, renderer, runner, p -> "en",
            PermissionSnapshotCache.disabled(), () -> null, () -> List.of(player));

        service.dispatch(player, new UiMsg.Open(new com.lkjmc.common.ui.kernel.MenuRoute("local")));

        assertFalse(renderer.models.stream().anyMatch(model -> model.phase() instanceof RoutePhase.Loading));
        assertTrue(renderer.models.stream().anyMatch(model -> model.phase() instanceof RoutePhase.Loaded));
    }

    @Test
    void transferClosesInventorySendsMessageAndDelegates() {
        var transfers = new ArrayList<String>();
        var fixture = fixture(new BindingRegistry(List.of()), transfers);
        var model = com.lkjmc.common.ui.kernel.UiModel.root("s1");

        fixture.runner.run(fixture.player, new UiEffect.Transfer("hub"), model, fixture.service);

        assertTrue(UiTestFixtures.state(fixture.player).closed);
        assertEquals(List.of("hub"), transfers);
        assertEquals(1, UiTestFixtures.state(fixture.player).messages.size());
    }

    private static Fixture fixture(BindingRegistry registry) {
        return fixture(registry, new ArrayList<>());
    }

    private static Fixture fixture(BindingRegistry registry, List<String> transfers) {
        var player = UiTestFixtures.player();
        var scheduler = new UiTestFixtures.Scheduler();
        var daemon = new FakeDaemon();
        var renderer = new UiSessionServiceTest.RecordingRenderer();
        var runner = runner(scheduler, daemon, registry, transfers);
        var service = new UiSessionService(UiTestFixtures.docs(), renderer, (p, e, m, s) -> {},
            p -> "en", PermissionSnapshotCache.disabled(), () -> null, () -> List.of(player));
        return new Fixture(player, scheduler, daemon, renderer, runner, service);
    }

    private static UiEffectRunner runner(UiTestFixtures.Scheduler scheduler, FakeDaemon daemon,
                                         BindingRegistry registry, List<String> transfers) {
        var text = UiTestFixtures.text();
        var input = new UiTextInput(scheduler, text, p -> "en");
        return new UiEffectRunner(scheduler, Optional.of(daemon), registry, new UiStaleCache(), input,
            text, UiTestFixtures.catalog(), (player, target) -> transfers.add(target));
    }

    private static String plain(Component component) {
        var value = new StringBuilder();
        append(component, value);
        return value.toString();
    }

    private static void append(Component component, StringBuilder value) {
        if (component instanceof TextComponent text) { value.append(text.content()); }
        component.children().forEach(child -> append(child, value));
    }

    private record Fixture(Player player, UiTestFixtures.Scheduler scheduler, FakeDaemon daemon,
                           UiSessionServiceTest.RecordingRenderer renderer, UiEffectRunner runner,
                           UiSessionService service) {}

    private static final class FakeDaemon implements DaemonClient {
        final CompletableFuture<DaemonResponse> future = new CompletableFuture<>();
        DaemonRequest request;
        @Override public CompletableFuture<DaemonResponse> send(DaemonRequest request) {
            this.request = request;
            return future;
        }
    }

    private static final class LocalBinding implements MenuBinding {
        @Override public String id() { return "local"; }
        @Override public DaemonRequestPlan plan(BindingContext ctx) {
            return new DaemonRequestPlan("local", "local", "", ctx.params(), Map.of(), List.of());
        }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            return BindingResult.data(new RouteView.DetailView(List.of(), List.of(TextRef.literal("done"))));
        }
    }
}
