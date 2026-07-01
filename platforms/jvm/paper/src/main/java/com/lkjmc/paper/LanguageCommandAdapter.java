package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.UUID;
import org.bukkit.entity.Player;

public final class LanguageCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public LanguageCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public boolean set(Player player, String[] args) {
        if (args.length != 1 || !validLanguage(args[0])) {
            player.sendMessage(message(player, "command.usage", Map.of("usage", "/lang <en|ja>")));
            return true;
        }
        var instanceId = System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId),
            "player.settings.set",
            Map.of(
                "playerUuid", player.getUniqueId().toString(),
                "name", player.getName(),
                "language", args[0].toLowerCase()
            )
        )).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> {
            if (response.ok()) {
                plugin.localeService().updateFromResponse(player, response.body());
            }
            player.sendMessage(message(player, response.ok() ? "language.saved" : "language.failed", Map.of()));
        })),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }

    private static boolean validLanguage(String value) {
        return value.equalsIgnoreCase("en") || value.equalsIgnoreCase("ja");
    }
}
