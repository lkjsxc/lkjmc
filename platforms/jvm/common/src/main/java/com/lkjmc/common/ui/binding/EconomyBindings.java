package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

public final class EconomyBindings {
    private EconomyBindings() {}

    public static List<MenuBinding> bindings() {
        return List.of(new Shop(), new Kits(), new Votes(), new Daily());
    }

    record ShopItem(String id, String titleKey, String category, long price, String kind,
                    boolean available, String reason, String material, long amount) {}

    static List<ShopItem> shopItems(JsonObject body, String binding) {
        var values = new ArrayList<ShopItem>();
        for (var value : Jsons.array(body, "items", binding)) {
            var row = Jsons.elementObject(value, binding);
            var delivery = optionalObject(row, "delivery");
            var material = Jsons.optionalString(delivery, "material");
            values.add(new ShopItem(Jsons.string(row, "id", binding), Jsons.string(row, "titleKey", binding),
                Jsons.string(row, "category", binding), Jsons.integer(row, "pricePoints", binding),
                Jsons.string(row, "deliveryKind", binding), Jsons.bool(row, "deliveryAvailable", binding),
                Jsons.string(row, "disabledReason", binding), material.isBlank() ? "BARRIER" : material,
                optionalLong(delivery, "amount")));
        }
        return values;
    }

    private static JsonObject optionalObject(JsonObject object, String field) {
        var element = object.get(field);
        return element == null || element.isJsonNull() ? null : element.getAsJsonObject();
    }

    private static long optionalLong(JsonObject object, String field) {
        var element = object == null ? null : object.get(field);
        return element == null || element.isJsonNull() ? 0 : element.getAsLong();
    }

    private static EntryView shopRow(ShopItem item, long balance) {
        var affordable = balance >= item.price();
        var reason = item.reason().isBlank() ? "menu.disabled.shop-delivery" : item.reason();
        DocumentAction action = !item.available() ? Views.disabled(reason) : !affordable
            ? Views.disabled("menu.disabled.shop-afford")
            : Views.command("buy " + item.id());
        var role = action instanceof DocumentAction.Disabled ? ItemRole.DISABLED : ItemRole.ACTION;
        return Views.entry(item.material(), Views.key(item.titleKey()),
            List.of(Views.lit(item.category()), Views.lit(item.price()), Views.lit(item.amount()),
                Views.key(role == ItemRole.DISABLED ? ((DocumentAction.Disabled) action).reasonKey()
                    : "menu.shop.buy.lore")), role, action);
    }

    private static List<FrameSlot> categories(List<ShopItem> items, String selected, Map<String, String> params) {
        var values = new ArrayList<FrameSlot>();
        var categories = items.stream().map(ShopItem::category).distinct().sorted().limit(6).toList();
        values.add(category(10, "all", selected, params));
        for (int i = 0; i < categories.size(); i++) {
            values.add(category(11 + i, categories.get(i), selected, params));
        }
        return values;
    }

    private static FrameSlot category(int slot, String category, String selected, Map<String, String> params) {
        var active = category.equals(selected);
        var action = active ? new DocumentAction.None() : Views.open("shop", Map.of("category", category));
        return Views.slot(slot, active ? "LIME_STAINED_GLASS_PANE" : "YELLOW_STAINED_GLASS_PANE",
            Views.lit(category), List.of(), active ? ItemRole.INFO : ItemRole.NAVIGATION, action, params);
    }

    private static final class Shop extends BasicBinding {
        Shop() { super("shop", "daemon", List.of("player.shop.list", "player.points.balance")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            Jsons.string(body, "playerUuid", id());
            var balance = Jsons.integer(body, "balance", id());
            var items = shopItems(body, id());
            if (items.isEmpty()) { return BindingResult.empty(); }
            var selected = ctx.param("category").orElse("all");
            var entries = items.stream().filter(item -> selected.equals("all") || selected.equals(item.category()))
                .sorted(Comparator.comparing(ShopItem::category).thenComparing(ShopItem::id))
                .map(item -> shopRow(item, balance)).toList();
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, List.of(Views.lit(balance)),
                    categories(items, selected, ctx.params())));
        }
    }

    private static final class Kits extends BasicBinding {
        Kits() { super("kits", "daemon", List.of("player.kit.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var entries = new ArrayList<EntryView>();
            for (var value : Jsons.array(body, "kits", id())) {
                var row = Jsons.elementObject(value, id());
                var kit = Jsons.string(row, "id", id());
                entries.add(Views.entry("IRON_SWORD", Views.key(Jsons.string(row, "titleKey", id())),
                    List.of(Views.lit(Jsons.integer(row, "rewardPoints", id())),
                        Views.lit(Jsons.integer(row, "cooldownHours", id())), Views.key("menu.kits.claim.lore")),
                    ItemRole.ACTION, Views.command("kit claim " + kit)));
            }
            entries.sort(Comparator.comparing(entry -> ((com.lkjmc.common.ui.kernel.TextRef.Key) entry.name()).key()));
            return entries.isEmpty() ? BindingResult.empty() : Views.data(new RouteView.ListView(entries, Views.keys("menu.kits.info.lore")));
        }
    }

    private static final class Votes extends BasicBinding {
        Votes() { super("votes", "daemon", List.of("player.vote.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var rows = new ArrayList<JsonObject>();
            for (var value : Jsons.array(body, "links", id())) { rows.add(Jsons.elementObject(value, id())); }
            rows.sort(Comparator.comparingLong(row -> Jsons.integer(row, "sortOrder", id())));
            var entries = rows.stream().map(row -> Views.entry("PAPER", Views.key(Jsons.string(row, "titleKey", id())),
                List.of(Views.lit(Jsons.string(row, "url", id())), Views.key("menu.votes.open.lore")),
                ItemRole.ACTION, Views.command("vote " + Jsons.string(row, "id", id())))).toList();
            return entries.isEmpty() ? BindingResult.empty() : Views.data(new RouteView.ListView(entries, Views.keys("menu.votes.info.lore")));
        }
    }

    private static final class Daily extends BasicBinding {
        Daily() { super("daily", "daemon", List.of("player.daily.status")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var claimed = Jsons.bool(body, "claimedToday", id());
            var points = Jsons.integer(body, "points", id());
            var action = claimed ? Views.disabled("menu.disabled.daily-claimed") : Views.command("daily");
            var role = claimed ? ItemRole.DISABLED : ItemRole.ACTION;
            var entry = Views.entry(claimed ? "GRAY_DYE" : "SUNFLOWER",
                Views.key(claimed ? "menu.daily.claimed-today" : "menu.daily.claim"),
                List.of(Views.lit(points), Views.key(claimed ? "menu.daily.claimed-today.lore" : "menu.daily.claim.lore")),
                role, action);
            return Views.data(new RouteView.ListView(List.of(entry), Views.keys("menu.daily.info.lore")));
        }
    }
}
