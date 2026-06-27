package com.lkjmc.paper;

import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.player.AsyncPlayerChatEvent;

final class MenuTextInputService implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final Map<UUID, Pending> pending = new ConcurrentHashMap<>();

    MenuTextInputService(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    void start(Player player, String promptKey, String commandPrefix) {
        pending.put(player.getUniqueId(), new Pending(commandPrefix));
        player.closeInventory();
        player.sendMessage(message(player, promptKey));
        player.sendMessage(message(player, "menu.input.cancel.lore"));
    }

    @EventHandler
    public void onChat(AsyncPlayerChatEvent event) {
        var input = pending.remove(event.getPlayer().getUniqueId());
        if (input == null) {
            return;
        }
        event.setCancelled(true);
        var text = event.getMessage() == null ? "" : event.getMessage().trim();
        plugin.scheduler().runPlayer(event.getPlayer(), () -> handle(event.getPlayer(), input, text));
    }

    private void handle(Player player, Pending input, String text) {
        if (text.equalsIgnoreCase("cancel")) {
            player.sendMessage(message(player, "menu.input.cancelled"));
            return;
        }
        if (text.isBlank() || text.contains(" ")) {
            player.sendMessage(message(player, "menu.input.invalid"));
            return;
        }
        player.performCommand(input.commandPrefix() + " " + text);
    }

    private String message(Player player, String key) {
        return renderer.render(player.locale().toLanguageTag(), key, Map.of());
    }

    private record Pending(String commandPrefix) {}
}
