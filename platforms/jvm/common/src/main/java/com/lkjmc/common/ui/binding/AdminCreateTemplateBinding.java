package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.List;

final class AdminCreateTemplateBinding extends BasicBinding {
    AdminCreateTemplateBinding() {
        super("admin-server-create-template", "daemon", List.of("instance.create.plan"));
    }

    @Override
    public BindingResult decode(JsonObject body, BindingContext ctx) {
        var startable = Jsons.bool(body, "startable", id());
        var kind = ctx.param("kind").orElse(Jsons.optionalString(body, "kind"));
        var template = "velocity".equals(kind) ? "velocity-modern" : "paper-survival";
        var diagnostics = body.has("diagnostics") && body.get("diagnostics").isJsonArray()
            ? body.getAsJsonArray("diagnostics").size() : 0;
        var slot = template(startable, template, diagnostics, ctx);
        return Views.data(new RouteView.CustomView(id(), List.of(slot), List.of(Views.lit(kind))));
    }

    private FrameSlot template(boolean startable, String template, int diagnostics, BindingContext ctx) {
        var enabled = ctx.permissions().createServer() && startable;
        var action = enabled ? new com.lkjmc.common.ui.document.DocumentAction.Input(
            "menu.admin.input.server-create", "lkjmc server create " + template)
            : Views.disabled(ctx.permissions().createServer() ? "menu.disabled.server-create-plan"
                : "menu.disabled.admin-permission");
        var lore = startable ? "menu.admin.server.template.default.lore" : "menu.disabled.server-create-plan";
        return Views.slot(22, enabled ? "OAK_SIGN" : "BARRIER",
            Views.key("menu.admin.server.template.default"), List.of(Views.key(lore), Views.lit(diagnostics)),
            enabled ? ItemRole.ACTION : ItemRole.DISABLED, action, ctx.params());
    }
}
