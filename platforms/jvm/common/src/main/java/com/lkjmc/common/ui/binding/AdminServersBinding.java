package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.List;
import java.util.Map;

final class AdminServersBinding extends BasicBinding {
    AdminServersBinding() {
        super("admin-servers", "daemon", List.of("instance.list"));
    }

    @Override
    public BindingResult decode(JsonObject body, BindingContext ctx) {
        var rows = ServerRow.list(body, id());
        var entries = rows.stream().map(row -> Views.entry(row.joinable() ? "GREEN_WOOL"
                : row.healthy() ? "YELLOW_WOOL" : "ORANGE_WOOL", Views.lit(row.id()),
            List.of(Views.lit(row.summary()), Views.key("menu.server-list.action.lore")),
            ItemRole.NAVIGATION, Views.open("admin-server-detail", Map.of("serverId", row.id())))).toList();
        return Views.data(new RouteView.ListView(entries, Views.keys("menu.admin.servers.lore"),
            List.of(create(ctx))));
    }

    private FrameSlot create(BindingContext ctx) {
        var enabled = ctx.permissions().createServer();
        return Views.keyedSlot(40, "NAME_TAG", "menu.admin.server.create",
            enabled ? ItemRole.ACTION : ItemRole.DISABLED,
            enabled ? Views.open("admin-server-create-kind") : Views.disabled("menu.disabled.admin-permission"),
            ctx.params());
    }
}
