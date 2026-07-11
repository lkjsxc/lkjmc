package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

public final class AdventureBinding extends BasicBinding {
    public AdventureBinding() {
        super("adventures", "daemon", List.of("adventure.catalog.list"));
    }

    @Override
    public BindingResult decode(JsonObject body, BindingContext ctx) {
        var entries = new ArrayList<EntryView>();
        for (var value : Jsons.array(body, "adventures", id())) {
            var row = Jsons.elementObject(value, id());
            var adventure = Jsons.string(row, "id", id());
            var title = Jsons.string(row, "titleKey", id());
            var icon = Jsons.string(row, "iconMaterial", id());
            var price = Jsons.integer(row, "pricePoints", id());
            var party = Jsons.integer(row, "maxPartySize", id());
            var enabled = Jsons.bool(row, "enabled", id());
            DocumentAction action = enabled && "end-expedition".equals(adventure)
                ? Views.open("adventures-end-confirm") : Views.disabled("menu.disabled.adventures");
            var available = !(action instanceof DocumentAction.Disabled);
            entries.add(Views.entry(icon, Views.key(title),
                List.of(Views.lit(price), Views.lit(party), Views.key(available
                    ? "menu.adventures.end.lore" : "menu.disabled.adventures")),
                available ? ItemRole.ACTION : ItemRole.DISABLED, action));
        }
        entries.sort(Comparator.comparing(entry -> entry.material() + entry.name().toString()));
        return entries.isEmpty() ? BindingResult.empty()
            : Views.data(new RouteView.ListView(entries, Views.keys("menu.adventures.lore")));
    }
}
