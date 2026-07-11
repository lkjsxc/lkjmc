package com.lkjmc.paper;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.Map;
import java.util.UUID;
import org.bukkit.Material;
import org.bukkit.entity.Player;
import org.bukkit.inventory.ItemStack;
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
        plugin.daemon().ifPresentOrElse(client -> client.send(request(command, body)).thenAccept(response ->
            plugin.scheduler().runPlayer(player, () -> handle(player, kind, response.ok(), response.body(),
                response.error().map(error -> error.code()).orElse("")))
        ), () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private void handle(Player player, String kind, boolean ok, JsonObject body, String errorCode) {
        if (kind.equals("shop.list")) {
            player.sendMessage(message(player, "shop.list.count", Map.of(
                "count", Integer.toString(DaemonJson.arraySize(body, "items"))
            )));
            return;
        }
        if (!ok) {
            complete(player, purchaseFailureKey(errorCode));
            return;
        }
        switch (purchaseAction(body)) {
            case REPLAY -> complete(player, "shop.purchase.replayed");
            case CONTAINED -> complete(player, "shop.purchase.delivery-contained");
            case DELIVER -> deliver(player, body);
        }
    }

    static PurchaseAction purchaseAction(JsonObject body) {
        if (DaemonJson.bool(body, "duplicate")) {
            return PurchaseAction.REPLAY;
        }
        return DaemonJson.bool(body, "refundable") || adventure(body)
            ? PurchaseAction.DELIVER : PurchaseAction.CONTAINED;
    }

    static String transferOutcome(boolean intentAccepted) {
        return intentAccepted ? "shop.purchase.transfer-pending" : "shop.purchase.delivery-contained";
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

    private void deliver(Player player, JsonObject body) {
        if (adventure(body)) {
            transferAdventure(player, body);
            return;
        }
        var delivery = DaemonJson.object(body, "delivery");
        if (delivery.isEmpty()) {
            complete(player, "shop.purchase.delivery-contained");
            return;
        }
        var material = Material.matchMaterial(DaemonJson.string(delivery.get(), "material").orElse(""));
        var amount = DaemonJson.integer(delivery.get(), "amount").orElse(0L).intValue();
        if (material == null || amount < 1 || amount > material.getMaxStackSize()) {
            complete(player, "shop.purchase.delivery-contained");
        } else if (canFit(player, material, amount)
            && player.getInventory().addItem(new ItemStack(material, amount)).isEmpty()) {
            complete(player, "shop.purchase.ok");
        } else {
            complete(player, "shop.purchase.delivery-contained");
        }
    }

    private void refund(Player player, JsonObject body) {
        var correlation = DaemonJson.string(body, "correlationId").orElse("");
        if (correlation.isBlank() || !DaemonJson.bool(body, "refundable")) {
            complete(player, "shop.purchase.delivery-contained");
            return;
        }
        plugin.daemon().ifPresentOrElse(client -> client.send(request("player.shop.refund", Map.of(
            "playerUuid", player.getUniqueId().toString(), "correlationId", correlation,
            "reason", "delivery-failed"
        ))).thenAccept(response -> plugin.scheduler().runPlayer(player, () -> complete(player,
            response.ok() && DaemonJson.bool(response.body(), "refunded")
                ? "shop.purchase.delivery-refunded" : "shop.purchase.delivery-contained"
        ))), () -> complete(player, "shop.purchase.delivery-contained"));
    }

    private void transferAdventure(Player player, JsonObject body) {
        var target = DaemonJson.string(body, "targetServer").orElse("");
        if (target.isBlank()) {
            complete(player, "shop.purchase.delivery-contained");
            return;
        }
        requestIntent(player, target, true);
        DaemonJson.array(body, "participants").ifPresent(participants -> {
            for (JsonElement entry : participants) {
                if (entry.isJsonObject()) {
                    var uuid = DaemonJson.string(entry.getAsJsonObject(), "playerUuid").orElse("");
                    var member = plugin.getServer().getPlayer(UUID.fromString(uuid));
                    if (member != null && !member.getUniqueId().equals(player.getUniqueId())) {
                        requestIntent(member, target, false);
                    }
                }
            }
        });
    }

    private void requestIntent(Player player, String target, boolean report) {
        var body = Map.<String, Object>of("playerUuid", player.getUniqueId().toString(),
            "playerName", player.getName(), "temporaryInstanceId", target);
        plugin.daemon().ifPresentOrElse(client -> client.send(request("temporary.transfer.intent", body))
            .whenComplete((response, error) -> plugin.scheduler().runPlayer(player, () -> {
                if (error == null && response != null && response.ok()) {
                    player.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
                        ProfileTransferMessages.transferRequest(target));
                    if (report) complete(player, transferOutcome(true));
                } else if (report) {
                    complete(player, transferOutcome(false));
                }
            })), () -> {
                if (report) complete(player, transferOutcome(false));
            });
    }

    private boolean canFit(Player player, Material material, int amount) {
        var capacity = 0;
        for (var stack : player.getInventory().getStorageContents()) {
            if (stack == null || stack.getType().isAir()) capacity += material.getMaxStackSize();
            else if (stack.getType() == material) capacity += material.getMaxStackSize() - stack.getAmount();
        }
        return capacity >= amount;
    }

    private static boolean adventure(JsonObject body) {
        return DaemonJson.object(body, "delivery")
            .flatMap(delivery -> DaemonJson.string(delivery, "executor"))
            .map(executor -> executor.equals("adventure") || executor.equals("adventure-end-expedition"))
            .orElse(false);
    }

    private static DaemonRequest request(String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body);
    }

    private void complete(Player player, String key) {
        var text = message(player, key, Map.of());
        player.sendMessage(text);
        player.sendActionBar(net.kyori.adventure.text.Component.text(text));
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(plugin.localeService().locale(player), key, values);
    }

    enum PurchaseAction { DELIVER, CONTAINED, REPLAY }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
