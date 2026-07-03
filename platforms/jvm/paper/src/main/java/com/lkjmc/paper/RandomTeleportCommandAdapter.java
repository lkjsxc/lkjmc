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
        var profile = "overworld";
        var confirmed = false;
        if (args.length == 1 && args[0].equalsIgnoreCase("confirm")) {
            confirmed = true;
        } else if (args.length >= 1) {
            profile = args[0].toLowerCase(java.util.Locale.ROOT);
            if (!profile.equals("overworld") && !profile.equals("nether") && !profile.equals("end")) {
                usage(player);
                return true;
            }
            if (args.length == 2 && args[1].equalsIgnoreCase("confirm")) {
                confirmed = true;
            } else if (args.length > 1) {
                usage(player);
                return true;
            }
        }
        return service.start(player, profile, confirmed);
    }

    RandomTeleportService service() { return service; }

    private void usage(Player player) {
        player.sendMessage(message(player, "command.usage", Map.of("usage", "/rtp [overworld|nether|end] [confirm]")));
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }
}
