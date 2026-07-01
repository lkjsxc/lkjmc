package com.lkjmc.paper;

import com.lkjmc.common.menu.AdminServerDynamicMenus;
import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuSpec;
import com.lkjmc.common.menu.MenuState;
import com.lkjmc.common.menu.ServerMenuEntry;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;

final class AdminServerMenuLoader {
    private final MenuDataGateway data;
    private final AdminMenuLoader admin;
    private final AdminServerCreatePlanner planner;

    AdminServerMenuLoader(MenuDataGateway data, AdminMenuLoader admin, AdminServerCreatePlanner planner) {
        this.data = data;
        this.admin = admin;
        this.planner = planner;
    }

    CompletableFuture<MenuSpec> load(Player player, MenuState state) {
        var permissions = admin.permissions(player);
        return switch (state.current().value()) {
            case "admin-servers" -> data.servers(player)
                .thenApply(entries -> AdminServerDynamicMenus.servers(entries, permissions));
            case "admin-server-detail" -> data.servers(player)
                .thenApply(entries -> AdminServerDynamicMenus.detail(selected(state, entries), permissions));
            case "admin-server-stop-confirm" -> CompletableFuture.completedFuture(
                AdminServerDynamicMenus.confirm(id(state), "stop", "instance.stop"));
            case "admin-server-restart-confirm" -> CompletableFuture.completedFuture(
                AdminServerDynamicMenus.confirm(id(state), "restart", "instance.restart"));
            case "admin-server-delete-confirm" -> CompletableFuture.completedFuture(
                AdminServerDynamicMenus.confirm(id(state), "delete", "instance.delete"));
            case "admin-server-create-kind" -> CompletableFuture.completedFuture(
                AdminServerDynamicMenus.createKind(permissions));
            case "admin-server-create-template" -> CompletableFuture.completedFuture(
                AdminServerDynamicMenus.createTemplate(param(state, "kind"), permissions));
            case "admin-server-create-confirm" -> createConfirm(player, state, permissions);
            default -> CompletableFuture.failedFuture(new IllegalArgumentException("not an admin server route"));
        };
    }

    private CompletableFuture<MenuSpec> createConfirm(Player player, MenuState state, com.lkjmc.common.menu.AdminMenuPermissions permissions) {
        var kind = param(state, "kind");
        var template = param(state, "template");
        var id = paramOr(state, "id", AdminServerDynamicMenus.generatedId(template));
        return planner.plan(player, id, kind, template)
            .thenApply(plan -> AdminServerDynamicMenus.createConfirm(kind, template, id, permissions,
                plan.startable(), plan.diagnostics()));
    }

    static boolean handles(MenuId id) {
        return switch (id.value()) {
            case "admin-servers", "admin-server-detail", "admin-server-stop-confirm",
                "admin-server-restart-confirm", "admin-server-delete-confirm", "admin-server-create-kind",
                "admin-server-create-template", "admin-server-create-confirm" -> true;
            default -> false;
        };
    }

    private static ServerMenuEntry selected(MenuState state, List<ServerMenuEntry> entries) {
        var id = id(state);
        return entries.stream().filter(entry -> entry.id().equals(id)).findFirst()
            .orElse(new ServerMenuEntry(id.isBlank() ? "unknown" : id, "unknown", "unknown", "unknown", false, null));
    }

    private static String id(MenuState state) {
        return param(state, "id");
    }

    private static String param(MenuState state, String key) {
        return state.route().params().getOrDefault(key, "");
    }

    private static String paramOr(MenuState state, String key, String fallback) {
        return state.route().params().getOrDefault(key, fallback);
    }
}
