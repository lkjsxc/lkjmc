package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.List;

final class AdminAuditBinding extends BasicBinding {
    AdminAuditBinding() {
        super("admin-audit", "daemon", List.of("admin.audit.tail"));
    }

    @Override
    public BindingResult decode(JsonObject body, BindingContext ctx) {
        if (!ctx.permissions().admin()) { return BindingResult.denied(); }
        var entries = new ArrayList<EntryView>();
        for (var value : Jsons.array(body, "events", id())) {
            var row = Jsons.elementObject(value, id());
            var actor = Jsons.string(row, "actorKind", id()) + ":" + Jsons.string(row, "actorId", id());
            var target = Jsons.string(row, "targetKind", id()) + ":" + Jsons.string(row, "targetId", id());
            var result = Jsons.string(row, "result", id());
            entries.add(Views.entry("PAPER", Views.lit(Jsons.string(row, "action", id())),
                List.of(Views.lit(actor), Views.lit(target), Views.lit(result)),
                ItemRole.INFO, new com.lkjmc.common.ui.document.DocumentAction.None()));
        }
        return entries.isEmpty() ? BindingResult.empty()
            : Views.data(new RouteView.ListView(entries, Views.keys("menu.admin.audit.lore")));
    }
}
