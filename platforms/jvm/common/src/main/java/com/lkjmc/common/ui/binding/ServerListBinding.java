package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

public final class ServerListBinding extends BasicBinding {
    public ServerListBinding() {
        super("server-list", "daemon", List.of("instance.list"));
    }

    @Override
    public BindingResult decode(JsonObject body, BindingContext ctx) {
        var rows = ServerRow.list(body, id());
        if (rows.isEmpty()) {
            return BindingResult.empty();
        }
        var entries = rows.stream().map(row -> entry(row, ctx.permissions())).toList();
        return Views.data(new RouteView.ListView(entries, Views.keys("menu.server-list.info.lore")));
    }

    static EntryView entry(ServerRow row, PermissionsView permissions) {
        var action = action(row, permissions);
        var role = action instanceof DocumentAction.Disabled ? ItemRole.DISABLED : ItemRole.ACTION;
        var lore = new ArrayList<>(List.of(Views.lit(row.kind()), Views.lit(row.observedState()),
            Views.lit(row.playerCount())));
        lore.add(action instanceof DocumentAction.Disabled disabled
            ? Views.key(disabled.reasonKey()) : Views.key("menu.server-list.action.lore"));
        return Views.entry(material(row), Views.lit(row.id() + " " + row.desiredState()), lore, role, action);
    }

    static DocumentAction action(ServerRow row, PermissionsView permissions) {
        if (row.joinable() && !permissions.stopServer()) {
            return new DocumentAction.Transfer(row.id());
        }
        return switch (row.desiredState()) {
            case "suspended" -> Views.daemon("instance.wake.request",
                Map.of("targetInstanceId", row.id()), "wake.queued", "wake.failed", true);
            case "stopped" -> permissions.startServer()
                ? Views.command("lkjmc server start " + row.id())
                : Views.disabled("menu.disabled.server-start-permission");
            case "running" -> running(row, permissions);
            case "starting" -> Views.disabled("menu.disabled.server-starting");
            default -> Views.disabled("menu.disabled.server-actions");
        };
    }

    private static DocumentAction running(ServerRow row, PermissionsView permissions) {
        if (permissions.stopServer()) {
            return row.playerCount() == 0 ? Views.command("lkjmc server stop " + row.id())
                : Views.disabled("menu.disabled.server-occupied");
        }
        return row.joinable() ? new DocumentAction.Transfer(row.id()) : Views.disabled(row.disabledReason());
    }

    private static String material(ServerRow row) {
        if (row.healthy()) {
            return "LIME_DYE";
        }
        return switch (row.desiredState()) {
            case "running", "starting" -> "YELLOW_DYE";
            case "suspended" -> "BLUE_DYE";
            default -> "GRAY_DYE";
        };
    }
}
