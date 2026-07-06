package com.lkjmc.paper;

import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.ui.kernel.MenuRoute;
import com.lkjmc.paper.ui.UiEntrypoints;
import java.util.Map;
import org.bukkit.entity.Player;

public final class RandomTeleportCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final UiEntrypoints entrypoints;
    private final RandomTeleportService service;

    public RandomTeleportCommandAdapter(
        LkjmcPaperPlugin plugin,
        MessageRenderer renderer,
        UiEntrypoints entrypoints
    ) {
        this.plugin = plugin;
        this.renderer = renderer;
        this.entrypoints = entrypoints;
        this.service = new RandomTeleportService(plugin, renderer);
    }

    public boolean handle(Player player, String[] args) {
        var parsed = parse(player, args);
        if (parsed == null) {
            return true;
        }
        if (paidProfile(parsed.profile()) && !parsed.confirmed()) {
            entrypoints.openDeep(player, route(parsed.profile()));
            return true;
        }
        return service.start(player, parsed.profile(), parsed.confirmed());
    }

    RandomTeleportService service() { return service; }

    private Parsed parse(Player player, String[] args) {
        var profile = "overworld";
        var confirmed = false;
        if (args.length == 1 && args[0].equalsIgnoreCase("confirm")) {
            confirmed = true;
        } else if (args.length >= 1) {
            profile = args[0].toLowerCase(java.util.Locale.ROOT);
            if (!profile.equals("overworld") && !paidProfile(profile)) {
                usage(player);
                return null;
            }
            if (args.length == 2 && args[1].equalsIgnoreCase("confirm")) {
                confirmed = true;
            } else if (args.length > 1) {
                usage(player);
                return null;
            }
        }
        return new Parsed(profile, confirmed);
    }

    private MenuRoute route(String profile) {
        return new MenuRoute("random-teleport-" + profile + "-confirm", Map.of(
            "profileId", profile,
            "serverId", instanceId()
        ));
    }

    private boolean paidProfile(String profile) {
        return profile.equals("nether") || profile.equals("end");
    }

    private void usage(Player player) {
        player.sendMessage(message(player, "command.usage", Map.of("usage", "/rtp [overworld|nether|end] [confirm]")));
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }

    private record Parsed(String profile, boolean confirmed) {}
}
