package com.lkjmc.common.ui.binding;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;

import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import com.lkjmc.common.ui.kernel.TextRef;
import java.util.Map;
import org.junit.jupiter.api.Test;

final class BindingActionTableTest {
    private final BindingRegistry registry = BindingRegistry.standard();

    @Test
    void serverRowsChooseTransferCommandOrDisabledReason() {
        var publicRows = list("server-list", "instance-list.json",
            BindingTestSupport.ctx(Map.of(), PermissionsView.none(), LocalData.empty()));
        assertInstanceOf(DocumentAction.Transfer.class, byLiteral(publicRows, "survival running").action());
        var disabled = assertInstanceOf(DocumentAction.Disabled.class, byLiteral(publicRows, "starting starting").action());
        assertEquals("menu.disabled.server-starting", disabled.reasonKey());

        var adminRows = list("server-list", "instance-list.json", BindingTestSupport.ctx());
        var command = assertInstanceOf(DocumentAction.Command.class,
            byLiteral(adminRows, "survival running").action());
        assertEquals("lkjmc server stop survival", command.value());
    }

    @Test
    void shopRowsChoosePurchaseUnaffordableOrUnavailable() {
        var view = list("shop", "shop-combined.json", BindingTestSupport.ctx());
        var buy = assertInstanceOf(DocumentAction.Daemon.class, byKey(view, "shop.item.food-bread-16").action());
        assertEquals("player.shop.purchase", buy.command());
        var delivery = assertInstanceOf(DocumentAction.Disabled.class,
            byKey(view, "shop.item.block-stone-64").action());
        assertEquals("menu.disabled.shop-delivery", delivery.reasonKey());
        var afford = assertInstanceOf(DocumentAction.Disabled.class,
            byKey(view, "shop.item.mineral-iron-ingot-8").action());
        assertEquals("menu.disabled.shop-afford", afford.reasonKey());
    }

    @Test
    void settingsRowsCarryCurrentStateAndToggleBody() {
        var view = detail("settings", "player-settings-get.json", BindingTestSupport.ctx());
        var hud = slot(view, 22).metadata().payload();
        assertEquals("daemon", hud.get("type"));
        assertEquals("hud", hud.get("body.settingKey"));
        assertEquals("hud.disabled", hud.get("ok"));
        assertEquals("hud.enabled", ((TextRef.Key) slot(view, 22).lore().getFirst()).key());
        var token = slot(view, 24).metadata().payload();
        assertEquals("menu-token", token.get("body.settingKey"));
        assertEquals("hotbar.menu.enabled", token.get("ok"));
        assertEquals("hotbar.menu.disabled", ((TextRef.Key) slot(view, 24).lore().getFirst()).key());
    }

    @Test
    void docsDirectorySearchAndFileUseLocalDataOnly() {
        var local = BindingTestSupport.ctx(Map.of("path", "guide"), PermissionsView.all(), BindingTestSupport.local());
        var directory = list("docs-directory", "empty.json", local);
        var open = assertInstanceOf(DocumentAction.Open.class, directory.entries().getFirst().action());
        assertEquals("docs-file", open.route());

        var searchCtx = BindingTestSupport.ctx(Map.of("query", "admin"), PermissionsView.all(), BindingTestSupport.local());
        var search = list("docs-search", "empty.json", searchCtx);
        assertEquals("Admin Guide", ((TextRef.Literal) search.entries().getFirst().name()).value());

        var fileCtx = BindingTestSupport.ctx(Map.of("path", "guide/start.md", "page", "0"),
            PermissionsView.all(), BindingTestSupport.local());
        var file = custom("docs-file", "empty.json", fileCtx);
        assertEquals("Start Guide", ((TextRef.Literal) slot(file, 22).name()).value());
        assertEquals("disabled", slot(file, 21).metadata().payload().get("type"));
    }

    private RouteView.ListView list(String binding, String fixture, BindingContext ctx) {
        return (RouteView.ListView) ((BindingResult.Data) registry.require(binding)
            .decode(BindingTestSupport.json(fixture), ctx)).view();
    }

    private RouteView.DetailView detail(String binding, String fixture, BindingContext ctx) {
        return (RouteView.DetailView) ((BindingResult.Data) registry.require(binding)
            .decode(BindingTestSupport.json(fixture), ctx)).view();
    }

    private RouteView.CustomView custom(String binding, String fixture, BindingContext ctx) {
        return (RouteView.CustomView) ((BindingResult.Data) registry.require(binding)
            .decode(BindingTestSupport.json(fixture), ctx)).view();
    }

    private EntryView byKey(RouteView.ListView view, String key) {
        return view.entries().stream().filter(entry -> entry.name() instanceof TextRef.Key ref
            && ref.key().equals(key)).findFirst().orElseThrow();
    }

    private EntryView byLiteral(RouteView.ListView view, String value) {
        return view.entries().stream().filter(entry -> entry.name() instanceof TextRef.Literal ref
            && ref.value().equals(value)).findFirst().orElseThrow();
    }

    private FrameSlot slot(RouteView.DetailView view, int slot) {
        return view.slots().stream().filter(value -> value.slot() == slot).findFirst().orElseThrow();
    }

    private FrameSlot slot(RouteView.CustomView view, int slot) {
        return view.slots().stream().filter(value -> value.slot() == slot).findFirst().orElseThrow();
    }
}
