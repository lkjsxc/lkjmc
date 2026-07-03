package com.lkjmc.paper;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.Map;
import java.util.Optional;
import org.bukkit.Material;
import org.bukkit.inventory.ItemStack;
import java.util.UUID;
import org.bukkit.entity.Player;

public final class ShopCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public ShopCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public boolean list(Player player) {
        return send(player, "player.shop.list", Map.of(), "shop.list");
    }

    public boolean buy(Player player, String[] args) {
        if (args.length != 1) {
            player.sendMessage(message(player, "command.usage", Map.of("usage", "/buy <item>")));
            return true;
        }
        return send(player, "player.shop.purchase", Map.of(
            "playerUuid", player.getUniqueId().toString(), "name", player.getName(),
            "itemId", args[0], "correlationId", UUID.randomUUID().toString()
        ), "shop.purchase");
    }

    private boolean send(Player player, String command, Map<String, Object> body, String kind) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> {
            var message = result(player, kind, response.ok(), response.body(),
                response.error().map(error -> error.code()).orElse(""));
            player.sendMessage(message);
            if (kind.equals("shop.purchase")) {
                player.sendActionBar(net.kyori.adventure.text.Component.text(message));
            }
        })), 
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String result(Player player, String kind, boolean ok, JsonObject body, String errorCode) {
        if (kind.equals("shop.list")) {
            var count = DaemonJson.arraySize(body, "items");
            return message(player, "shop.list.count", Map.of("count", Integer.toString(count)));
        }
        if (!ok) {
            return message(player, purchaseFailureKey(errorCode), Map.of());
        }
        if (deliver(player, body)) {
            return message(player, "shop.purchase.ok", Map.of());
        }
        refund(player, body, "delivery-failed");
        return message(player, "shop.purchase.delivery-refunded", Map.of());
    }

    static String purchaseFailureKey(String code) {
        return switch (code) {
            case "shop.insufficient_points" -> "shop.purchase.insufficient";
            case "shop.item_not_found" -> "shop.purchase.not-found";
            case "shop.unsupported_delivery" -> "shop.purchase.unsupported-delivery";
            case "daemon.auth_failed", "auth.failed", "admin.denied" -> "shop.purchase.auth-failed";
            case "shop.invalid_material" -> "shop.purchase.invalid-material";
            case "shop.delivery_refunded" -> "shop.purchase.delivery-refunded";
            case "database.error", "database.not_configured", "database.unavailable" -> "shop.purchase.database";
            case "menu.schema_mismatch", "schema.mismatch" -> "shop.purchase.schema-mismatch";
            case "adventure.duplicate_active" -> "shop.purchase.duplicate-adventure";
            case "adventure.disabled", "adventure.error", "temporary.error" -> "shop.purchase.adventure-failed";
            default -> "shop.purchase.failed";
        };
    }

    private void refund(Player player, JsonObject body, String reason) {
        var correlation = DaemonJson.string(body, "correlationId").orElse("");
        if (correlation.isBlank()) {
            return;
        }
        plugin.daemon().ifPresent(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "player.shop.refund",
            Map.of("playerUuid", player.getUniqueId().toString(), "correlationId", correlation, "reason", reason)
        )));
    }

    private boolean deliver(Player player, JsonObject body) {
        if (!body.has("delivery") || !body.get("delivery").isJsonObject()) {
            return false;
        }
        var delivery = body.getAsJsonObject("delivery");
        var executor = DaemonJson.string(delivery, "executor").orElse("");
        if ("adventure".equals(executor) || "adventure-end-expedition".equals(executor)) {
            return deliverAdventure(body);
        }
        if (!"minecraft-item".equals(executor)) {
            return false;
        }
        var material = Material.matchMaterial(DaemonJson.string(delivery, "material").orElse(""));
        if (material == null) {
            return false;
        }
        var amount = DaemonJson.integer(delivery, "amount").orElse(1L).intValue();
        var leftovers = player.getInventory().addItem(new ItemStack(material, Math.max(1, Math.min(64, amount))));
        leftovers.values().forEach(item -> player.getWorld().dropItemNaturally(player.getLocation(), item));
        return true;
    }

    private boolean deliverAdventure(JsonObject body) {
        var target = DaemonJson.string(body, "targetServer").orElse("");
        if (target.isBlank() || !body.has("participants") || !body.get("participants").isJsonArray()) {
            return false;
        }
        for (JsonElement element : body.getAsJsonArray("participants")) {
            if (!element.isJsonObject()) {
                continue;
            }
            var uuid = DaemonJson.string(element.getAsJsonObject(), "playerUuid").flatMap(this::parseUuid);
            uuid.map(plugin.getServer()::getPlayer).ifPresent(player -> requestIntent(player, target));
        }
        return true;
    }

    private void requestIntent(Player player, String target) {
        var body = Map.<String, Object>of(
            "playerUuid", player.getUniqueId().toString(), "playerName", player.getName(), "temporaryInstanceId", target
        );
        plugin.daemon().ifPresent(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), "temporary.transfer.intent", body
        )).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> {
            if (response.ok()) {
                player.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
                    ProfileTransferMessages.transferRequest(target));
            }
        })));
    }

    private Optional<UUID> parseUuid(String value) {
        try {
            return Optional.of(UUID.fromString(value));
        } catch (IllegalArgumentException ignored) {
            return Optional.empty();
        }
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
