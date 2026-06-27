package com.lkjmc.paper;

import java.time.Duration;
import org.bukkit.entity.Player;

final class InventorySyncService {
    private final LkjmcPaperPlugin plugin;
    private final HotbarMenuTokenService tokens;

    InventorySyncService(LkjmcPaperPlugin plugin, HotbarMenuTokenService tokens) {
        this.plugin = plugin;
        this.tokens = tokens;
    }

    void repairNow(Player player) {
        plugin.scheduler().runPlayer(player, () -> repair(player));
    }

    void repairWithDelays(Player player) {
        repairNow(player);
        plugin.scheduler().runPlayerLater(player, () -> repair(player), Duration.ofMillis(100));
        plugin.scheduler().runPlayerLater(player, () -> repair(player), Duration.ofMillis(500));
    }

    private void repair(Player player) {
        for (int index = 0; index < player.getInventory().getSize(); index++) {
            if (index != HotbarMenuTokenService.SLOT && tokens.isToken(player.getInventory().getItem(index))) {
                player.getInventory().setItem(index, null);
            }
        }
        player.getInventory().setItem(HotbarMenuTokenService.SLOT, tokens.create(player));
        player.updateInventory();
    }
}
