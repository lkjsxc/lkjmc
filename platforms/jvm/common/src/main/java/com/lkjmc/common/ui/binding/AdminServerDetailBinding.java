package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

final class AdminServerDetailBinding extends BasicBinding {
    AdminServerDetailBinding() {
        super("admin-server-detail", "daemon", List.of("instance.list"));
    }

    @Override
    public BindingResult decode(JsonObject body, BindingContext ctx) {
        var target = ctx.param("serverId").orElse("");
        return ServerRow.list(body, id()).stream().filter(row -> row.id().equals(target)).findFirst()
            .<BindingResult>map(row -> Views.data(detail(row, ctx))).orElseGet(BindingResult::empty);
    }

    private RouteView.DetailView detail(ServerRow row, BindingContext ctx) {
        var slots = new ArrayList<FrameSlot>();
        slots.add(action(19, "LIME_WOOL", "menu.admin.server.start", "instance.start", row.id(),
            ctx.permissions().startServer(), ctx));
        slots.add(confirm(20, "ORANGE_WOOL", "menu.admin.server.stop",
            "admin-server-stop-confirm", row.id(), ctx.permissions().stopServer(), ctx));
        slots.add(confirm(21, "ANVIL", "menu.admin.server.restart",
            "admin-server-restart-confirm", row.id(), ctx.permissions().restartServer(), ctx));
        slots.add(action(22, "PAPER", "menu.admin.audit.tail", "instance.logs", row.id(),
            ctx.permissions().listServers(), ctx));
        slots.add(confirm(24, "BARRIER", "menu.admin.server.delete",
            "admin-server-delete-confirm", row.id(), ctx.permissions().deleteServer(), ctx));
        return new RouteView.DetailView(slots, List.of(Views.lit(row.id()), Views.lit(row.summary())));
    }

    private FrameSlot action(int slot, String material, String key, String command, String server,
                             boolean enabled, BindingContext ctx) {
        var ok = "instance.start".equals(command) ? "server.start.requested" : "menu.admin.audit.tail";
        var action = enabled ? Views.daemon(command, Map.of("id", server), ok, "daemon.unavailable", true)
            : Views.disabled("menu.disabled.admin-permission");
        return Views.keyedSlot(slot, material, key, enabled ? ItemRole.ACTION : ItemRole.DISABLED,
            action, ctx.params());
    }

    private FrameSlot confirm(int slot, String material, String key, String route, String server,
                              boolean enabled, BindingContext ctx) {
        var action = enabled ? Views.open(route, Map.of("serverId", server))
            : Views.disabled("menu.disabled.admin-permission");
        return Views.keyedSlot(slot, material, key, enabled ? ItemRole.ACTION : ItemRole.DISABLED,
            action, ctx.params());
    }
}
