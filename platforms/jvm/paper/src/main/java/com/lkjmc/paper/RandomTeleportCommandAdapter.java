package com.lkjmc.paper;

import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import org.bukkit.entity.Player;

public final class RandomTeleportCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final RandomTeleportService service;

    public RandomTeleportCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
        this.service = new RandomTeleportService(plugin, renderer);
    }

    public boolean handle(Player player, String[] args) {
        if (args.length > 1 || (args.length == 1 && !args[0].equalsIgnoreCase("confirm"))) {
            player.sendMessage(message(player, "command.usage", Map.of("usage", "/rtp [confirm]")));
            return true;
        }
        return service.start(player);
    }

    RandomTeleportService service() {
        return service;
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }
}
