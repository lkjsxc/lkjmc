package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

public final class ProfileBindings {
    private ProfileBindings() {}

    public static List<MenuBinding> bindings() {
        return List.of(new Profile(), new Achievements(), new Directory(), new Detail());
    }

    record Achievement(String id, String title, String description, String category, String icon,
                       long current, long required, String state, boolean hidden,
                       boolean claimable, boolean claimed, int rewards) {}

    static List<Achievement> achievements(JsonObject body, String binding) {
        var values = new ArrayList<Achievement>();
        for (var value : Jsons.array(body, "achievements", binding)) {
            var row = Jsons.elementObject(value, binding);
            values.add(new Achievement(Jsons.string(row, "id", binding), Jsons.string(row, "titleKey", binding),
                Jsons.string(row, "descriptionKey", binding), Jsons.string(row, "categoryPath", binding),
                Jsons.string(row, "iconMaterial", binding), Jsons.integer(row, "current", binding),
                Jsons.integer(row, "required", binding), Jsons.string(row, "state", binding),
                Jsons.bool(row, "hidden", binding), Jsons.bool(row, "claimable", binding),
                Jsons.bool(row, "rewardClaimed", binding), Jsons.array(row, "rewards", binding).size()));
        }
        return values;
    }

    static List<Achievement> visible(JsonObject body, String binding) {
        return achievements(body, binding).stream().filter(a -> !a.hidden() || !"locked".equals(a.state())).toList();
    }

    private static EntryView directory(String path) {
        return Views.entry("claimable".equals(path) ? "EMERALD" : "BOOK", Views.lit(path),
            Views.keys("menu.achievements.directory.lore"), ItemRole.NAVIGATION,
            Views.open("achievement-directory", Map.of("path", path)));
    }

    private static EntryView achievement(Achievement value) {
        return Views.entry(value.icon(), Views.key(value.title()),
            List.of(Views.key(value.description()), Views.lit(value.state())), ItemRole.NAVIGATION,
            Views.open("achievement-detail", Map.of("id", value.id())));
    }

    private static int rank(Achievement value) {
        return switch (value.state()) { case "claimable" -> 0; case "in-progress" -> 1; case "claimed" -> 2; default -> 3; };
    }

    private static String progress(Achievement value) {
        var filled = (int) Math.min(10, Math.max(0, value.current()) * 10 / Math.max(1, value.required()));
        return "[" + "#".repeat(filled) + "-".repeat(10 - filled) + "] "
            + value.current() + "/" + value.required();
    }

    private static final class Profile extends BasicBinding {
        Profile() { super("profile", "daemon", List.of("player.points.balance", "player.achievements.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            Jsons.string(body, "playerUuid", id());
            var balance = Jsons.integer(body, "balance", id());
            var claimed = achievements(body, id()).stream().filter(a -> a.claimed() || "claimed".equals(a.state())).count();
            var slots = List.of(
                Views.keyedSlot(20, "EMERALD", "menu.profile.points", ItemRole.INFO,
                    new com.lkjmc.common.ui.document.DocumentAction.None(), ctx.params(), "menu.profile.points.lore"),
                Views.keyedSlot(22, "DIAMOND", "menu.profile.achievements", ItemRole.NAVIGATION,
                    Views.open("achievements"), ctx.params(), "menu.profile.achievements.lore"),
                Views.keyedSlot(24, "CLOCK", "menu.profile.hud", ItemRole.NAVIGATION,
                    Views.open("settings"), ctx.params(), "menu.profile.hud.lore"));
            return Views.data(new RouteView.DetailView(slots, List.of(Views.lit(balance), Views.lit(claimed))));
        }
    }

    private static final class Achievements extends BasicBinding {
        Achievements() { super("achievements", "daemon", List.of("player.achievements.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var values = visible(body, id());
            var dirs = new ArrayList<String>();
            if (values.stream().anyMatch(Achievement::claimable)) { dirs.add("claimable"); }
            values.stream().map(Achievement::category).distinct().sorted().forEach(dirs::add);
            var entries = dirs.stream().map(ProfileBindings::directory).toList();
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, Views.keys("menu.achievements.info.lore")));
        }
    }

    private static final class Directory extends BasicBinding {
        Directory() { super("achievement-directory", "daemon", List.of("player.achievements.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var path = ctx.param("path").orElse("claimable");
            var entries = visible(body, id()).stream()
                .filter(a -> "claimable".equals(path) ? a.claimable() : path.equals(a.category()))
                .sorted(Comparator.comparingInt(ProfileBindings::rank).thenComparing(Achievement::id))
                .map(ProfileBindings::achievement).toList();
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, List.of(Views.lit(path))));
        }
    }

    private static final class Detail extends BasicBinding {
        Detail() { super("achievement-detail", "daemon", List.of("player.achievements.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var target = ctx.param("id").orElse("");
            return visible(body, id()).stream().filter(a -> a.id().equals(target)).findFirst()
                .<BindingResult>map(a -> Views.data(detail(a, ctx))).orElseGet(BindingResult::empty);
        }
        private RouteView.DetailView detail(Achievement a, BindingContext ctx) {
            var action = a.claimable() ? Views.daemon("player.achievement.claim",
                Map.of("achievementId", a.id()), "achievements.reward.claimed", "achievements.reward.failed", true)
                : Views.disabled("menu.achievements.disabled.not-claimable");
            var role = a.claimable() ? ItemRole.ACTION : ItemRole.DISABLED;
            var slots = List.of(
                Views.slot(22, a.icon(), Views.key(a.title()),
                    List.of(Views.key(a.description()), Views.lit(a.category()), Views.lit(progress(a)),
                        Views.lit(a.state()), Views.lit(a.rewards())), ItemRole.INFO,
                    new com.lkjmc.common.ui.document.DocumentAction.None(), ctx.params()),
                Views.keyedSlot(31, a.claimable() ? "EMERALD" : "BARRIER", a.claimable()
                    ? "menu.achievements.claim" : "menu.achievements.disabled.not-claimable",
                    role, action, ctx.params(), a.claimable() ? "menu.achievements.claim.lore"
                        : "menu.achievements.disabled.not-claimable"));
            return new RouteView.DetailView(slots, List.of(Views.lit(a.id())));
        }
    }
}
