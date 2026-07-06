package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;

final class AdminSectionBinding extends BasicBinding {
    AdminSectionBinding(String id, String... commands) {
        super(id, "daemon", Arrays.asList(commands));
    }

    @Override
    public BindingResult decode(JsonObject body, BindingContext ctx) {
        validate(body);
        var slots = switch (id()) {
            case "admin-config" -> config(ctx);
            case "admin-economy" -> economy(ctx);
            case "admin-moderation" -> moderation(ctx);
            case "admin-security" -> security(ctx);
            case "admin-web" -> web(ctx);
            default -> List.<FrameSlot>of();
        };
        return Views.data(new RouteView.DetailView(slots, Views.keys(infoKey())));
    }

    private void validate(JsonObject body) {
        switch (id()) {
            case "admin-config", "admin-web" -> {
                Jsons.string(body, "daemon", id());
                Jsons.integer(body, "startedAtUnixSeconds", id());
                Jsons.integer(body, "uptimeSeconds", id());
                var database = Jsons.object(body, "database", id());
                Jsons.bool(database, "configured", id());
                Jsons.bool(database, "connected", id());
                var runtime = Jsons.object(body, "runtime", id());
                Jsons.string(runtime, "adapter", id());
            }
            case "admin-economy" -> EconomyBindings.shopItems(body, id());
            case "admin-moderation" -> SocialBindings.reports(body, id());
            case "admin-security" -> {
                Jsons.array(body, "roles", id());
                Jsons.bool(body, "configured", id());
            }
            default -> { }
        }
    }

    private List<FrameSlot> config(BindingContext ctx) {
        return List.of(
            action(20, "COMPASS", "menu.admin.doctor", "lkjmc doctor", ctx.permissions().status(), ctx),
            action(22, "REDSTONE", "menu.admin.reload", "lkjmc config reload", ctx.permissions().reload(), ctx),
            input(24, "CLOCK", "menu.admin.restart-warn", "menu.admin.input.restart-warn",
                "lkjmc restart warn", ctx.permissions().reload(), ctx));
    }

    private List<FrameSlot> economy(BindingContext ctx) {
        return List.of(
            action(20, "EMERALD", "menu.admin.seed-defaults", "lkjmc economy seed-defaults",
                ctx.permissions().economy(), ctx),
            input(22, "CHEST", "menu.admin.shop-upsert", "menu.admin.input.shop-upsert",
                "lkjmc shop item upsert", false, ctx),
            input(24, "OAK_SIGN", "menu.admin.announce", "menu.admin.input.announce",
                "announce", ctx.permissions().announce(), ctx));
    }

    private List<FrameSlot> moderation(BindingContext ctx) {
        return List.of(
            slot(19, "REDSTONE_TORCH", "menu.reports.title", ctx.permissions().reports()
                ? Views.open("reports") : Views.disabled("menu.disabled.admin-permission"), ctx.permissions().reports(), ctx),
            input(20, "PAPER", "menu.admin.warn", "menu.admin.input.warn", "warn", ctx.permissions().warn(), ctx),
            input(21, "WRITABLE_BOOK", "menu.admin.note", "menu.admin.input.note", "note", ctx.permissions().warn(), ctx),
            input(22, "IRON_AXE", "menu.admin.ban", "menu.admin.input.ban", "ban", ctx.permissions().ban(), ctx),
            input(23, "BARRIER", "menu.admin.mute", "menu.admin.input.mute", "mute", ctx.permissions().mute(), ctx),
            action(24, "GOLDEN_SHOVEL", "menu.admin.claims", "claim list", ctx.permissions().claim(), ctx));
    }

    private List<FrameSlot> security(BindingContext ctx) {
        return List.of(
            action(19, "BOOK", "menu.admin.roles", "lkjmc admin role list", ctx.permissions().admin(), ctx),
            input(20, "LIME_DYE", "menu.admin.grant", "menu.admin.input.grant", "lkjmc admin grant",
                ctx.permissions().admin(), ctx),
            input(21, "PLAYER_HEAD", "menu.admin.inspect", "menu.admin.input.inspect", "lkjmc admin inspect",
                ctx.permissions().admin(), ctx),
            input(22, "RED_DYE", "menu.admin.revoke", "menu.admin.input.revoke", "lkjmc admin revoke",
                ctx.permissions().admin(), ctx),
            action(23, "NETHER_STAR", "menu.admin.token.status", "lkjmc security daemon-token status",
                ctx.permissions().admin(), ctx),
            action(25, "TNT", "menu.admin.token.rotate", "lkjmc security daemon-token rotate",
                ctx.permissions().admin(), ctx));
    }

    private List<FrameSlot> web(BindingContext ctx) {
        return List.of(action(22, "OAK_SIGN", "menu.admin.web.status", "lkjmc status",
            ctx.permissions().status(), ctx));
    }

    private FrameSlot action(int slot, String material, String key, String command,
                             boolean enabled, BindingContext ctx) {
        return slot(slot, material, key, enabled ? Views.command(command)
            : Views.disabled("menu.disabled.admin-permission"), enabled, ctx);
    }

    private FrameSlot input(int slot, String material, String key, String prompt,
                            String prefix, boolean enabled, BindingContext ctx) {
        var action = enabled ? new DocumentAction.Input(prompt, prefix)
            : Views.disabled("menu.disabled.admin-permission");
        return slot(slot, material, key, action, enabled, ctx);
    }

    private FrameSlot slot(int slot, String material, String key, DocumentAction action,
                           boolean enabled, BindingContext ctx) {
        return Views.keyedSlot(slot, material, key, enabled ? ItemRole.ACTION : ItemRole.DISABLED,
            action, ctx.params());
    }

    private String infoKey() {
        return switch (id()) {
            case "admin-config" -> "menu.admin.config.lore";
            case "admin-economy" -> "menu.admin.economy.lore";
            case "admin-moderation" -> "menu.admin.moderation.lore";
            case "admin-security" -> "menu.admin.security.lore";
            default -> "menu.admin.web.lore";
        };
    }
}
