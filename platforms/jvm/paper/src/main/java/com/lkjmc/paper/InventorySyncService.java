package com.lkjmc.paper;

import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import java.time.Duration;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.entity.Player;

final class InventorySyncService {
    private final LkjmcPaperPlugin plugin;
    private final HotbarMenuTokenService tokens;
    private final Optional<DaemonClient> daemon;

    InventorySyncService(LkjmcPaperPlugin plugin, HotbarMenuTokenService tokens, Optional<DaemonClient> daemon) {
        this.plugin = plugin;
        this.tokens = tokens;
        this.daemon = daemon == null ? Optional.empty() : daemon;
    }

    void repairNow(Player player) {
        plugin.scheduler().runPlayer(player, () -> repair(player));
    }

    void repairWithDelays(Player player) {
        refreshSetting(player);
        repairNow(player);
        plugin.scheduler().runPlayerLater(player, () -> repair(player), Duration.ofMillis(100));
        plugin.scheduler().runPlayerLater(player, () -> repair(player), Duration.ofMillis(500));
    }

    void setTokenEnabled(Player player, boolean enabled) {
        tokens.setEnabled(player, enabled);
        repairNow(player);
    }

    private void refreshSetting(Player player) {
        daemon.ifPresent(client -> client.send(new DaemonRequest(
            UUID.randomUUID(),
            new DaemonActor("paper-plugin", player.getName()),
            "player.settings.get",
            Map.of("playerUuid", player.getUniqueId().toString())
        )).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> {
            if (response.ok() && response.body().has("menuEnabled")) {
                tokens.setEnabled(player, response.body().get("menuEnabled").getAsBoolean());
                repair(player);
            }
        })));
    }

    private void repair(Player player) {
        for (int index = 0; index < player.getInventory().getSize(); index++) {
            if (index != HotbarMenuTokenService.SLOT && tokens.isToken(player.getInventory().getItem(index))) {
                player.getInventory().setItem(index, null);
            }
        }
        if (tokens.enabled(player)) {
            player.getInventory().setItem(HotbarMenuTokenService.SLOT, tokens.create(player));
        } else if (tokens.isToken(player.getInventory().getItem(HotbarMenuTokenService.SLOT))) {
            player.getInventory().setItem(HotbarMenuTokenService.SLOT, null);
        }
        player.updateInventory();
    }
}
