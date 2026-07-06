package com.lkjmc.paper.ui;

import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MiniMessageText;
import com.lkjmc.common.ui.binding.BindingRegistry;
import com.lkjmc.common.ui.document.MenuDocumentLoader;
import com.lkjmc.paper.LkjmcPaperPlugin;
import java.util.Objects;

public record UiWiring(
    UiSessionService sessions,
    UiInventoryListener inventoryListener,
    UiTextInput textInput,
    UiEntrypoints entrypoints
) {
    public static UiWiring create(LkjmcPaperPlugin plugin, UiTransferPort transfers) {
        Objects.requireNonNull(transfers, "transfers");
        var documents = MenuDocumentLoader.fromResources();
        var docs = DocBundle.load(UiWiring.class.getResourceAsStream("/lkjmc-docs-bundle.json"));
        var resolver = new LocaleResolver("en");
        var text = new UiText(new MiniMessageText(plugin.catalog(), resolver));
        var metadata = new UiMetadataCodec(plugin);
        var renderer = new UiRenderer(documents, metadata, text);
        var bindings = BindingRegistry.standard();
        documents.documents().stream().filter(document -> document.data() != null)
            .forEach(document -> bindings.require(document.data().binding()));
        var stale = new UiStaleCache();
        var input = new UiTextInput(plugin.scheduler(), text, player -> plugin.localeService().locale(player));
        var effects = new UiEffectRunner(plugin.scheduler(), plugin.daemon(), bindings,
            stale, input, text, plugin.catalog(), transfers);
        var sessions = new UiSessionService(documents, renderer, effects,
            player -> plugin.localeService().locale(player), plugin.adminGrants(), () -> docs,
            () -> plugin.getServer().getOnlinePlayers());
        var listener = new UiInventoryListener(sessions, metadata);
        var entrypoints = new UiEntrypoints(plugin.scheduler(), sessions);
        return new UiWiring(sessions, listener, input, entrypoints);
    }
}
