package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.UUID;
import org.bukkit.Material;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;

public final class ExchangeCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public ExchangeCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public boolean exchange(Player player, String[] args) {
        if (args.length != 2) {
            player.sendMessage(message(player, "exchange.usage", Map.of()));
            return true;
        }
        var material = Material.matchMaterial(args[0]);
        if (material == null || material.isAir()) {
            player.sendMessage(message(player, "exchange.invalid-material", Map.of()));
            return true;
        }
        var count = count(player, material);
        var amount = amount(args[1], count);
        if (amount <= 0) {
            player.sendMessage(message(player, "exchange.invalid-amount", Map.of()));
            return true;
        }
        if (count < amount) {
            player.sendMessage(message(player, "exchange.insufficient", Map.of("count", Long.toString(count))));
            return true;
        }
        if (plugin.daemon().isEmpty()) {
            player.sendMessage(message(player, "daemon.unavailable", Map.of()));
            return true;
        }
        try {
            remove(player, material, amount);
        } catch (IllegalArgumentException error) {
            player.sendMessage(message(player, "exchange.insufficient", Map.of("count", Long.toString(count))));
            return true;
        }
        var correlation = UUID.randomUUID();
        var body = Map.<String, Object>of(
            "playerUuid", player.getUniqueId().toString(),
            "name", player.getName(),
            "material", material.name(),
            "amount", amount,
            "correlationId", correlation.toString()
        );
        plugin.daemon().get().send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "player.exchange.commit", body
        )).whenComplete((response, error) -> plugin.scheduler().runPlayer(player, () -> {
            if (error != null || response == null || !response.ok()) {
                refund(player, material, amount);
                player.sendMessage(message(player, "exchange.commit-failed", Map.of()));
                return;
            }
            var message = success(player, material, response.body());
            player.sendMessage(message);
            player.sendActionBar(net.kyori.adventure.text.Component.text(message));
        }));
        return true;
    }

    private long amount(String text, long count) {
        if (text.equalsIgnoreCase("all")) {
            return count;
        }
        try {
            return Long.parseLong(text);
        } catch (NumberFormatException error) {
            return -1;
        }
    }

    private long count(Player player, Material material) {
        return ExchangeInventoryPlanner.count(player.getInventory().getStorageContents(), material);
    }

    private void remove(Player player, Material material, long amount) {
        player.getInventory().setStorageContents(ExchangeInventoryPlanner.remove(
            player.getInventory().getStorageContents(), material, amount));
    }

    private boolean refund(Player player, Material material, long amount) {
        var remaining = amount;
        while (remaining > 0) {
            var stackAmount = (int) Math.min(material.getMaxStackSize(), remaining);
            if (!player.getInventory().addItem(new ItemStack(material, stackAmount)).isEmpty()) {
                return false;
            }
            remaining -= stackAmount;
        }
        return true;
    }

    private String success(Player player, Material material, JsonObject body) {
        var amount = DaemonJson.integer(body, "amount").orElse(0L);
        var points = DaemonJson.integer(body, "pointsDelta").orElse(0L);
        return message(player, "exchange.ok", Map.of(
            "amount", Long.toString(amount),
            "material", material.name(),
            "points", Long.toString(points)
        ));
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
