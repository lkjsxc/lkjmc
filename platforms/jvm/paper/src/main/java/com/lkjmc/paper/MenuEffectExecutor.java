package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.daemon.DaemonResponse;
import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.MenuActionPayload;
import com.lkjmc.common.menu.MenuEffect;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.entity.Player;

final class MenuEffectExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageCatalog catalog;
    private final LocaleResolver resolver;
    private final Optional<DaemonClient> daemon;
    private final MenuInventoryAdapter menus;
    private final InventorySyncService sync;
    private final MenuTextInputService textInput;

    MenuEffectExecutor(LkjmcPaperPlugin plugin, MessageCatalog catalog, LocaleResolver resolver,
                       Optional<DaemonClient> daemon, MenuInventoryAdapter menus, InventorySyncService sync,
                       MenuTextInputService textInput) {
        this.plugin = plugin;
        this.catalog = catalog;
        this.resolver = resolver;
        this.daemon = daemon == null ? Optional.empty() : daemon;
        this.menus = menus;
        this.sync = sync;
        this.textInput = textInput;
    }

    void execute(Player player, MenuEffect effect) {
        switch (effect) {
            case MenuEffect.OpenRoute open -> menus.open(player, open.route());
            case MenuEffect.OpenPrevious ignored -> menus.openPrevious(player);
            case MenuEffect.CloseMenu ignored -> player.closeInventory();
            case MenuEffect.RefreshRoute ignored -> menus.refresh(player);
            case MenuEffect.RunPlayerCommand command -> runCommand(player, command.command());
            case MenuEffect.SendDaemonCommand command -> sendDaemon(player, command);
            case MenuEffect.TransferPlayer transfer -> player.sendMessage(render(player, "hub.unavailable"));
            case MenuEffect.SendMessage message -> player.sendMessage(render(player, message.key()));
            case MenuEffect.PromptText prompt -> textInput.start(player, prompt.promptKey(), prompt.commandPrefix());
            case MenuEffect.RenderLoadingThenRun loading -> execute(player, loading.effect());
            case MenuEffect.Noop ignored -> { }
        }
    }

    private void runCommand(Player player, String command) {
        player.performCommand(command);
    }

    private void sendDaemon(Player player, MenuEffect.SendDaemonCommand command) {
        if (daemon.isEmpty()) {
            player.sendMessage(render(player, "daemon.unavailable"));
            return;
        }
        var request = new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", player.getName()),
            command.command(), body(player, command.body()));
        daemon.get().send(request).whenComplete((response, error) -> plugin.scheduler().runPlayer(player, () -> {
            if (error != null || response == null || !response.ok()) {
                player.sendMessage(render(player, failureKey(command)));
            } else {
                handleSuccess(player, command, response);
                menus.refresh(player);
            }
        }));
    }

    private Map<String, Object> body(Player player, MenuActionPayload payload) {
        var body = new HashMap<String, Object>();
        body.put("playerUuid", player.getUniqueId().toString());
        body.put("name", player.getName());
        body.put("playerName", player.getName());
        payload.values().forEach((key, value) -> body.put(key, typed(value)));
        return body;
    }

    private Object typed(String value) {
        if ("true".equalsIgnoreCase(value) || "false".equalsIgnoreCase(value)) {
            return Boolean.parseBoolean(value);
        }
        return value;
    }

    private void handleSuccess(Player player, MenuEffect.SendDaemonCommand command, DaemonResponse response) {
        if (command.command().equals("player.settings.set")) {
            player.sendMessage(render(player, "language.saved"));
            return;
        }
        if (command.command().equals("instance.wake.request")) {
            var target = com.lkjmc.common.daemon.DaemonJson.string(response.body(), "targetServer").orElse("");
            if (!target.isBlank()) {
                player.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
                    ProfileTransferMessages.transferRequest(target));
                player.sendMessage(render(player, "wake.ready"));
            }
            return;
        }
        if (response.body().has("hudEnabled")) {
            player.sendMessage(render(player, response.body().get("hudEnabled").getAsBoolean() ? "hud.enabled" : "hud.disabled"));
        }
        if (response.body().has("menuEnabled")) {
            var enabled = response.body().get("menuEnabled").getAsBoolean();
            sync.setTokenEnabled(player, enabled);
            player.sendMessage(render(player, enabled ? "hotbar.menu.enabled" : "hotbar.menu.disabled"));
        }
    }

    private String failureKey(MenuEffect.SendDaemonCommand command) {
        if (command.command().equals("player.settings.set")) {
            return "language.failed";
        }
        if (command.command().equals("instance.wake.request")) {
            return "wake.failed";
        }
        if (command.body().value().contains("menu-token")) {
            return "hotbar.menu.failed";
        }
        if (command.body().value().contains("hud")) {
            return "hud.failed";
        }
        return "daemon.unavailable";
    }

    private String render(Player player, String key) {
        return catalog.render(resolver.resolve(Optional.of(player.locale().toLanguageTag())), key);
    }
}
