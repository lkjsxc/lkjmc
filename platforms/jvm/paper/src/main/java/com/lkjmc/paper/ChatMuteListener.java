package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import io.papermc.paper.event.player.AsyncChatEvent;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.TimeUnit;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;

public final class ChatMuteListener implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public ChatMuteListener(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @EventHandler(ignoreCancelled = true)
    public void onChat(AsyncChatEvent event) {
        if (!event.isAsynchronous() || plugin.daemon().isEmpty()) {
            return;
        }
        var player = event.getPlayer();
        var request = new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId()),
            "player.moderation.status",
            Map.of("playerUuid", player.getUniqueId().toString(), "playerName", player.getName())
        );
        try {
            var response = plugin.daemon().get().send(request).get(2, TimeUnit.SECONDS);
            var raw = response.body().getOrDefault("raw", "").toString();
            if (response.ok() && raw.contains("\"muted\":true")) {
                event.setCancelled(true);
                var reason = extract(raw, "muteReason");
                plugin.scheduler().runPlayer(player, () -> player.sendMessage(renderer.render(
                    player.locale().toLanguageTag(), "moderation.chat-denied", Map.of("reason", reason)
                )));
            }
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
        } catch (java.util.concurrent.ExecutionException | java.util.concurrent.TimeoutException ignored) {
            // Fail open so chat is not blocked by an unavailable daemon.
        }
    }

    private static String extract(String json, String key) {
        var needle = "\"" + key + "\":\"";
        var start = json.indexOf(needle);
        if (start < 0) {
            return "";
        }
        var valueStart = start + needle.length();
        var end = json.indexOf('"', valueStart);
        return end < 0 ? "" : json.substring(valueStart, end);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
