package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.List;
import java.util.Map;

public final class RandomTeleportBinding extends BasicBinding {
    public RandomTeleportBinding() {
        super("random-teleport", "daemon", List.of("player.random-teleport.quote"));
    }

    @Override
    public BindingResult decode(JsonObject body, BindingContext ctx) {
        var profile = Jsons.string(body, "profileId", id());
        Jsons.string(body, "targetEnvironment", id());
        var cost = Jsons.integer(body, "costPoints", id());
        var balance = Jsons.integer(body, "balance", id());
        Jsons.integer(body, "cooldownSeconds", id());
        var remaining = Jsons.integer(body, "cooldownRemainingSeconds", id());
        var min = Jsons.integer(body, "minRadius", id());
        var max = Jsons.integer(body, "maxRadius", id());
        var attempts = Jsons.integer(body, "maxAttempts", id());
        var confirm = Jsons.bool(body, "confirmationRequired", id());
        var enabled = Jsons.bool(body, "enabled", id());
        var afford = Jsons.bool(body, "canAfford", id());
        Jsons.array(body, "allowedWorlds", id());
        Jsons.array(body, "worldCandidates", id());
        var slots = List.of(info(ctx, cost, min, max, attempts), action(ctx, profile, cost,
            balance, remaining, confirm, enabled, afford), cancel(ctx));
        return Views.data(new RouteView.DetailView(slots, List.of(Views.lit(profile), Views.lit(balance))));
    }

    private FrameSlot info(BindingContext ctx, long cost, long min, long max, long attempts) {
        return Views.keyedSlot(13, "ENDER_PEARL", "menu.random-teleport.confirm", ItemRole.INFO,
            new DocumentAction.None(), ctx.params(), "menu.random-teleport.confirm.lore");
    }

    private FrameSlot action(BindingContext ctx, String profile, long cost, long balance, long remaining,
                             boolean confirm, boolean enabled, boolean afford) {
        if (remaining > 0) {
            return disabled(ctx, "menu.random-teleport.disabled.cooldown", Long.toString(remaining));
        }
        if (!afford) {
            return disabled(ctx, "menu.random-teleport.disabled.unaffordable", balance + "/" + cost);
        }
        if (!enabled) {
            return disabled(ctx, "menu.random-teleport.disabled.policy", profile);
        }
        var command = confirm ? "rtp " + profile + " confirm" : "rtp " + profile;
        return Views.keyedSlot(11, "LIME_WOOL", confirm ? "menu.confirm.yes" : "menu.random-teleport.start",
            ItemRole.SUCCESS, Views.command(command), ctx.params(), "menu.random-teleport.confirm.lore");
    }

    private FrameSlot disabled(BindingContext ctx, String reason, String detail) {
        return Views.slot(11, "GRAY_WOOL", Views.key("menu.confirm.yes"),
            List.of(Views.key(reason), Views.lit(detail)), ItemRole.DISABLED, Views.disabled(reason), ctx.params());
    }

    private FrameSlot cancel(BindingContext ctx) {
        return Views.keyedSlot(15, "RED_WOOL", "menu.confirm.no", ItemRole.NAVIGATION,
            new DocumentAction.Back(), ctx.params());
    }
}
