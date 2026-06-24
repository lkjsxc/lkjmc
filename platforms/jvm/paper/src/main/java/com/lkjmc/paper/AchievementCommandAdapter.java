package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.UUID;
import org.bukkit.entity.Player;

public final class AchievementCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public AchievementCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public boolean list(Player player) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId()),
            "player.achievements.list",
            Map.of("playerUuid", player.getUniqueId().toString())
        )).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(result(player, response.ok(), response.body().get("raw"))))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String result(Player player, boolean ok, Object raw) {
        if (!ok) {
            return message(player, "achievements.failed", Map.of());
        }
        var count = raw == null ? 0 : countIds(raw.toString());
        return message(player, "achievements.count", Map.of("count", Integer.toString(count)));
    }

    private static int countIds(String json) {
        var count = 0;
        var index = json.indexOf("\"id\":");
        while (index >= 0) {
            count++;
            index = json.indexOf("\"id\":", index + 5);
        }
        return count;
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
