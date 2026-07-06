package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.List;
import java.util.Map;

final class AdminCreateKindBinding extends BasicBinding {
    AdminCreateKindBinding() {
        super("admin-server-create-kind", "daemon", List.of("instance.create.plan"));
    }

    @Override
    public BindingResult decode(JsonObject body, BindingContext ctx) {
        var slots = List.of(kind(20, "GRASS_BLOCK", "paper", "menu.admin.server.kind.paper",
                "menu.admin.server.kind.paper.lore", ctx),
            kind(24, "NETHER_STAR", "velocity", "menu.admin.server.kind.velocity",
                "menu.admin.server.kind.velocity.lore", ctx));
        return Views.data(new RouteView.CustomView(id(), slots, Views.keys("menu.admin.server.kind.info")));
    }

    private FrameSlot kind(int slot, String material, String kind, String key, String lore,
                           BindingContext ctx) {
        var enabled = ctx.permissions().createServer();
        var action = enabled ? Views.open("admin-server-create-template", Map.of("kind", kind))
            : Views.disabled("menu.disabled.admin-permission");
        return Views.keyedSlot(slot, material, key, enabled ? ItemRole.NAVIGATION : ItemRole.DISABLED,
            action, ctx.params(), lore);
    }
}
