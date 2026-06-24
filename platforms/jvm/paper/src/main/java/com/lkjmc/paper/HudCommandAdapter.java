package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.UUID;
import net.kyori.adventure.text.Component;
import org.bukkit.entity.Player;

public final class HudCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public HudCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public boolean set(Player player, String[] args) {
        if (args.length != 1 || enabled(args[0]) == null) {
            player.sendMessage(message(player, "command.usage", Map.of("usage", "/hud <on|off>")));
            return true;
        }
        var enabled = enabled(args[0]);
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId()),
            "player.settings.hud",
            Map.of(
                "playerUuid", player.getUniqueId().toString(),
                "name", player.getName(),
                "enabled", enabled
            )
        )).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> preview(player, response.ok(), enabled))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private void preview(Player player, boolean ok, boolean enabled) {
        var key = ok ? (enabled ? "hud.enabled" : "hud.disabled") : "hud.failed";
        var text = message(player, key, Map.of());
        player.sendMessage(text);
        if (ok && enabled) {
            player.sendActionBar(Component.text(text));
        }
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }

    private static Boolean enabled(String value) {
        if (value.equalsIgnoreCase("on")) {
            return true;
        }
        if (value.equalsIgnoreCase("off")) {
            return false;
        }
        return null;
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
