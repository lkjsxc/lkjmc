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

    AdminServerMenuLoader(MenuDataGateway data, AdminMenuLoader admin) {
        this.data = data;
        this.admin = admin;
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
            default -> CompletableFuture.failedFuture(new IllegalArgumentException("not an admin server route"));
        };
    }

    static boolean handles(MenuId id) {
        return switch (id.value()) {
            case "admin-servers", "admin-server-detail", "admin-server-stop-confirm",
                "admin-server-restart-confirm", "admin-server-delete-confirm" -> true;
            default -> false;
        };
    }

    private static ServerMenuEntry selected(MenuState state, List<ServerMenuEntry> entries) {
        var id = id(state);
        return entries.stream().filter(entry -> entry.id().equals(id)).findFirst()
            .orElse(new ServerMenuEntry(id.isBlank() ? "unknown" : id, "unknown", "unknown", "unknown", false, null));
    }

    private static String id(MenuState state) {
        return state.route().params().getOrDefault("id", "");
    }
}
