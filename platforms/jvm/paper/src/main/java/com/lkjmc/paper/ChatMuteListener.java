package com.lkjmc.paper;

import com.lkjmc.common.i18n.MessageRenderer;
import io.papermc.paper.event.player.AsyncChatEvent;
import java.util.Map;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;

public final class ChatMuteListener implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final MuteSnapshotService mutes;

    public ChatMuteListener(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
        this.mutes = new MuteSnapshotService(plugin);
        this.mutes.start();
    }

    @EventHandler
    public void onJoin(PlayerJoinEvent event) {
        mutes.refresh(event.getPlayer());
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        mutes.remove(event.getPlayer());
    }

    @EventHandler(ignoreCancelled = true)
    public void onChat(AsyncChatEvent event) {
        var player = event.getPlayer();
        mutes.track(player);
        mutes.current(player.getUniqueId()).ifPresent(mute -> {
            event.setCancelled(true);
            plugin.scheduler().runPlayer(player, () -> player.sendMessage(renderer.render(
                plugin.localeService().locale(player),
                "moderation.chat-denied",
                Map.of("reason", mute.reason())
            )));
        });
    }
}
