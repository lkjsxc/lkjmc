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
        if (args.length != 2) return invalid(player, "exchange.usage");
        var material = Material.matchMaterial(args[0]);
        if (material == null || material.isAir()) return invalid(player, "exchange.invalid-material");
        var count = count(player, material);
        var amount = amount(args[1], count);
        if (amount <= 0) return invalid(player, "exchange.invalid-amount");
        if (count < amount) return invalid(player, "exchange.insufficient", Map.of("count", Long.toString(count)));
        if (plugin.daemon().isEmpty()) return invalid(player, "daemon.unavailable");
        try {
            remove(player, material, amount);
        } catch (IllegalArgumentException error) {
            return invalid(player, "exchange.insufficient", Map.of("count", Long.toString(count)));
        }
        var correlation = UUID.randomUUID();
        var body = Map.<String, Object>of("playerUuid", player.getUniqueId().toString(),
            "name", player.getName(), "material", material.name(), "amount", amount,
            "correlationId", correlation.toString());
        plugin.daemon().get().send(request("player.exchange.commit", body)).whenComplete((response, error) ->
            plugin.scheduler().runPlayer(player, () -> {
                if (error == null && response != null && response.ok()) {
                    success(player, material, response.body());
                } else {
                    reconcile(player, material, amount, correlation, error == null && response != null);
                }
            })
        );
        return true;
    }

    private void reconcile(Player player, Material material, long amount, UUID correlation, boolean definitive) {
        var body = Map.<String, Object>of("playerUuid", player.getUniqueId().toString(),
            "correlationId", correlation.toString());
        plugin.daemon().ifPresentOrElse(client -> client.send(request("player.exchange.reconcile", body))
            .whenComplete((response, error) -> plugin.scheduler().runPlayer(player, () -> {
                var found = error == null && response != null && response.ok();
                var absent = error == null && response != null && !response.ok()
                    && response.error().map(value -> value.code().equals("exchange.correlation_not_found")).orElse(false);
                switch (recovery(definitive && absent, found)) {
                    case COMMITTED -> success(player, material, response.body());
                    case RESTORE -> restore(player, material, amount);
                    case CONTAIN -> invalid(player, "exchange.commit-contained");
                }
            })), () -> invalid(player, "exchange.commit-contained"));
    }

    static Recovery recovery(boolean definitiveAbsence, boolean found) {
        if (found) return Recovery.COMMITTED;
        return definitiveAbsence ? Recovery.RESTORE : Recovery.CONTAIN;
    }

    private void restore(Player player, Material material, long amount) {
        if (refund(player, material, amount)) invalid(player, "exchange.commit-failed");
        else invalid(player, "exchange.commit-contained");
    }

    private long amount(String text, long count) {
        if (text.equalsIgnoreCase("all")) return count;
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
            if (!player.getInventory().addItem(new ItemStack(material, stackAmount)).isEmpty()) return false;
            remaining -= stackAmount;
        }
        return true;
    }

    private void success(Player player, Material material, JsonObject body) {
        var text = message(player, "exchange.ok", Map.of(
            "amount", Long.toString(DaemonJson.integer(body, "amount").orElse(0L)),
            "material", material.name(),
            "points", Long.toString(DaemonJson.integer(body, "pointsDelta").orElse(0L))));
        player.sendMessage(text);
        player.sendActionBar(net.kyori.adventure.text.Component.text(text));
    }

    private boolean invalid(Player player, String key) {
        return invalid(player, key, Map.of());
    }

    private boolean invalid(Player player, String key, Map<String, String> values) {
        player.sendMessage(message(player, key, values));
        return true;
    }

    private static DaemonRequest request(String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body);
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }

    enum Recovery { COMMITTED, RESTORE, CONTAIN }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
