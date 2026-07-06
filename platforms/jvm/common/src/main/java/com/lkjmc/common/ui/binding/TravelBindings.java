package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

public final class TravelBindings {
    private TravelBindings() {}

    public static List<MenuBinding> bindings() {
        return List.of(new Homes(), new HomeDetail(), new Warps(), new Teleports());
    }

    private record Place(String name, String serverId) {}

    private static List<Place> places(JsonObject body, String array, String name, String binding) {
        var values = new ArrayList<Place>();
        for (var value : Jsons.array(body, array, binding)) {
            var row = Jsons.elementObject(value, binding);
            var loc = Jsons.object(row, "location", binding);
            Jsons.string(loc, "world", binding);
            Jsons.integer(loc, "x", binding);
            Jsons.integer(loc, "y", binding);
            Jsons.integer(loc, "z", binding);
            values.add(new Place(Jsons.string(row, name, binding), Jsons.string(row, "serverId", binding)));
        }
        return values.stream().sorted(Comparator.comparing(Place::name)).toList();
    }

    private static EntryView home(Place place) {
        var valid = place.name().matches("[A-Za-z0-9_-]{1,32}");
        var action = valid ? Views.open("home-detail", Map.of("home", place.name()))
            : Views.disabled("menu.disabled.invalid-home-name");
        var role = valid ? ItemRole.ACTION : ItemRole.DISABLED;
        return Views.entry(valid ? "RED_BED" : "BARRIER", Views.lit(place.name()),
            List.of(Views.lit(place.serverId()), Views.key(valid ? "menu.homes.detail.lore"
                : "menu.disabled.invalid-home-name")), role, action);
    }

    private static EntryView warp(Place place) {
        return Views.entry("OAK_SIGN", Views.lit(place.name()),
            List.of(Views.lit(place.serverId()), Views.key("menu.travel.teleport.lore")),
            ItemRole.ACTION, Views.command("warp " + place.name()));
    }

    private static final class Homes extends BasicBinding {
        Homes() { super("homes", "daemon", List.of("player.home.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var entries = places(body, "homes", "home", id()).stream().map(TravelBindings::home).toList();
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, Views.keys("menu.homes.title")));
        }
    }

    private static final class HomeDetail extends BasicBinding {
        HomeDetail() { super("home-detail", "daemon", List.of("player.home.get")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            if (!Jsons.bool(body, "found", id())) { return BindingResult.empty(); }
            var loc = Jsons.object(body, "location", id());
            Jsons.string(loc, "world", id());
            Jsons.integer(loc, "x", id());
            Jsons.integer(loc, "y", id());
            Jsons.integer(loc, "z", id());
            var home = Jsons.string(body, "home", id());
            var server = Jsons.string(body, "serverId", id());
            var slots = List.of(
                Views.keyedSlot(20, "ENDER_PEARL", "menu.homes.teleport", ItemRole.ACTION,
                    Views.command("home " + home), ctx.params(), "menu.homes.teleport.lore"),
                Views.keyedSlot(22, "LIME_BED", "menu.homes.update", ItemRole.ACTION,
                    Views.open("home-update-confirm", Map.of("home", home)), ctx.params(), "menu.homes.update.lore"),
                Views.keyedSlot(24, "TNT", "menu.homes.delete", ItemRole.ACTION,
                    Views.open("home-delete-confirm", Map.of("home", home)), ctx.params(), "menu.homes.delete.lore"));
            return Views.data(new RouteView.DetailView(slots, List.of(Views.lit(home), Views.lit(server))));
        }
    }

    private static final class Warps extends BasicBinding {
        Warps() { super("warps", "daemon", List.of("player.warp.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var entries = places(body, "warps", "warp", id()).stream().map(TravelBindings::warp).toList();
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, Views.keys("menu.warps.title")));
        }
    }

    private static final class Teleports extends BasicBinding {
        Teleports() { super("teleports", "daemon", List.of("player.snapshot")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var entries = List.of(
                Views.keyed("ENDER_PEARL", "menu.teleports.request", Views.open("teleport-picker"),
                    ItemRole.NAVIGATION, "menu.teleports.request.lore"),
                Views.keyed("CHORUS_FRUIT", "menu.random-teleport.title",
                    Views.open("random-teleport-overworld"), ItemRole.NAVIGATION, "menu.random-teleport.lore"),
                Views.keyed("LIME_DYE", "menu.teleports.accept", Views.command("tpaccept"),
                    ItemRole.ACTION, "menu.teleports.accept.lore"));
            return Views.data(new RouteView.ListView(entries, Views.keys("menu.teleports.info.lore")));
        }
    }
}
