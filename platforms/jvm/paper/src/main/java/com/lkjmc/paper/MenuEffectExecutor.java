package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.MenuEffect;
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

    MenuEffectExecutor(LkjmcPaperPlugin plugin, MessageCatalog catalog, LocaleResolver resolver,
                       Optional<DaemonClient> daemon, MenuInventoryAdapter menus) {
        this.plugin = plugin;
        this.catalog = catalog;
        this.resolver = resolver;
        this.daemon = daemon == null ? Optional.empty() : daemon;
        this.menus = menus;
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
            case MenuEffect.RenderLoadingThenRun loading -> execute(player, loading.effect());
            case MenuEffect.Noop ignored -> { }
        }
    }

    private void runCommand(Player player, String command) {
        player.closeInventory();
        player.performCommand(command);
    }

    private void sendDaemon(Player player, MenuEffect.SendDaemonCommand command) {
        if (daemon.isEmpty()) {
            player.sendMessage(render(player, "daemon.unavailable"));
            return;
        }
        var request = new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", player.getName()),
            command.command(), Map.of("payload", command.body().value()));
        daemon.get().send(request).whenComplete((response, error) -> plugin.scheduler().runPlayer(player, () -> {
            if (error != null || response == null || !response.ok()) {
                player.sendMessage(render(player, "daemon.unavailable"));
            } else {
                menus.refresh(player);
            }
        }));
    }

    private String render(Player player, String key) {
        return catalog.render(resolver.resolve(Optional.of(player.locale().toLanguageTag())), key);
    }
}
