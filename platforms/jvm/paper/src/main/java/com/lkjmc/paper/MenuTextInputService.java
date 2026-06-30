package com.lkjmc.paper;

import com.lkjmc.common.i18n.MessageRenderer;
import java.time.Duration;
import java.time.Instant;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.AsyncPlayerChatEvent;
import org.bukkit.event.player.PlayerQuitEvent;

final class MenuTextInputService implements Listener {
    private static final Duration INPUT_TTL = Duration.ofSeconds(60);

    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final Map<UUID, Pending> pending = new ConcurrentHashMap<>();

    MenuTextInputService(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    void start(Player player, String promptKey, String commandPrefix) {
        var input = new Pending(commandPrefix, UUID.randomUUID(), Instant.now().plus(INPUT_TTL));
        pending.put(player.getUniqueId(), input);
        player.sendMessage(message(player, promptKey));
        player.sendMessage(message(player, "menu.input.cancel.lore"));
        plugin.scheduler().runPlayerLater(player, () -> expire(player, input), INPUT_TTL);
    }

    @EventHandler
    public void onChat(AsyncPlayerChatEvent event) {
        var input = pending.remove(event.getPlayer().getUniqueId());
        if (input == null) {
            return;
        }
        if (input.expired()) {
            plugin.scheduler().runPlayer(event.getPlayer(), () ->
                event.getPlayer().sendMessage(message(event.getPlayer(), "menu.input.expired")));
            return;
        }
        event.setCancelled(true);
        var text = event.getMessage() == null ? "" : event.getMessage().trim();
        plugin.scheduler().runPlayer(event.getPlayer(), () -> handle(event.getPlayer(), input, text));
    }

    @EventHandler
    public void onQuit(PlayerQuitEvent event) {
        pending.remove(event.getPlayer().getUniqueId());
    }

    private void expire(Player player, Pending input) {
        if (pending.remove(player.getUniqueId(), input)) {
            player.sendMessage(message(player, "menu.input.expired"));
        }
    }

    private void handle(Player player, Pending input, String text) {
        if (text.equalsIgnoreCase("cancel")) {
            player.sendMessage(message(player, "menu.input.cancelled"));
            return;
        }
        if (text.isBlank()) {
            player.sendMessage(message(player, "menu.input.invalid"));
            return;
        }
        player.performCommand(input.commandPrefix() + " " + text);
    }

    private String message(Player player, String key) {
        return renderer.render(player.locale().toLanguageTag(), key, Map.of());
    }

    private record Pending(String commandPrefix, UUID token, Instant expiresAt) {
        boolean expired() {
            return Instant.now().isAfter(expiresAt);
        }
    }
}
