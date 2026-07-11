package com.lkjmc.paper;

import org.bukkit.entity.Player;

final class InventorySyncService {
    private final HotbarMenuTokenService tokens;

    InventorySyncService(HotbarMenuTokenService tokens) {
        this.tokens = tokens;
    }

    void repair(Player player) {
        for (int slot = 0; slot < player.getInventory().getSize(); slot++) {
            if (slot != HotbarMenuTokenService.SLOT && tokens.isToken(player.getInventory().getItem(slot))) {
                player.getInventory().setItem(slot, null);
            }
        }
        player.getInventory().setItem(HotbarMenuTokenService.SLOT, tokens.create());
        player.updateInventory();
    }
}
