package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.menu.ShopMenuEntry;
import com.lkjmc.common.menu.ShopView;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.bukkit.Material;
import org.bukkit.entity.Player;

final class ShopMenuDataGateway {
    private final Optional<DaemonClient> daemon;

    ShopMenuDataGateway(Optional<DaemonClient> daemon) {
        this.daemon = daemon == null ? Optional.empty() : daemon;
    }

    CompletableFuture<ShopView> shop(Player player, String category) {
        var balance = request(player, "player.points.balance", Map.of("playerUuid", player.getUniqueId().toString()))
            .thenApply(body -> integer(body, "balance"));
        var items = request(player, "player.shop.list", Map.of()).thenApply(body -> items(body, 0));
        return balance.thenCombine(items, (amount, entries) -> new ShopView(amount, category,
            entries.stream().map(entry -> withAffordability(entry, amount)).toList()));
    }

    private CompletableFuture<JsonObject> request(Player player, String command, Map<String, Object> body) {
        if (daemon.isEmpty()) {
            return CompletableFuture.failedFuture(MenuDataException.missingDaemon());
        }
        var request = new com.lkjmc.common.daemon.DaemonRequest(UUID.randomUUID(),
            new DaemonActor("paper-plugin", player.getName()), command, body);
        return daemon.get().send(request).thenApply(response -> {
            if (!response.ok()) {
                throw MenuDataException.response(command, response);
            }
            return response.body();
        });
    }

    private static java.util.List<ShopMenuEntry> items(JsonObject body, long balance) {
        var entries = new ArrayList<ShopMenuEntry>();
        if (!body.has("items") || !body.get("items").isJsonArray()) {
            throw MenuDataException.schema("player.shop.list", "items");
        }
        for (var value : body.getAsJsonArray("items")) {
            if (value.isJsonObject()) {
                entries.add(entry(value.getAsJsonObject(), balance));
            }
        }
        return List.copyOf(entries);
    }

    private static ShopMenuEntry withAffordability(ShopMenuEntry entry, long balance) {
        var affordable = balance >= entry.pricePoints();
        return new ShopMenuEntry(entry.id(), entry.titleKey(), entry.category(), entry.material(), entry.amount(),
            entry.pricePoints(), entry.deliveryKind(), entry.deliveryAvailable(), affordable,
            affordable ? entry.disabledReason() : "menu.disabled.shop-afford");
    }

    private static ShopMenuEntry entry(JsonObject item, long balance) {
        var delivery = item.has("delivery") && item.get("delivery").isJsonObject() ? item.getAsJsonObject("delivery") : new JsonObject();
        var price = integer(item, "pricePoints");
        return new ShopMenuEntry(text(item, "id", "unknown"), text(item, "titleKey", "unknown"),
            text(item, "category", "misc"), text(delivery, "material", text(item, "material", "CHEST")),
            integer(delivery, "amount"), price, text(item, "deliveryKind", text(delivery, "executor", "")),
            available(item, delivery), balance >= price, disabled(item, delivery));
    }

    private static boolean available(JsonObject item, JsonObject delivery) {
        if (!bool(item, "deliveryAvailable")) { return false; }
        var executor = text(item, "deliveryKind", text(delivery, "executor", ""));
        return !"minecraft-item".equals(executor) || Material.matchMaterial(text(delivery, "material", "")) != null;
    }

    private static String disabled(JsonObject item, JsonObject delivery) {
        if (available(item, delivery)) { return text(item, "disabledReason", ""); }
        var executor = text(item, "deliveryKind", text(delivery, "executor", ""));
        if ("minecraft-item".equals(executor) && Material.matchMaterial(text(delivery, "material", "")) == null) {
            return "menu.disabled.shop-invalid-material";
        }
        return text(item, "disabledReason", "menu.disabled.shop-delivery");
    }

    private static long integer(JsonObject object, String key) {
        return object.has(key) && !object.get(key).isJsonNull() ? object.get(key).getAsLong() : 0;
    }

    private static boolean bool(JsonObject object, String key) {
        return object.has(key) && !object.get(key).isJsonNull() && object.get(key).getAsBoolean();
    }

    private static String text(JsonObject object, String key, String fallback) {
        return object.has(key) && !object.get(key).isJsonNull() ? object.get(key).getAsString() : fallback;
    }
}
