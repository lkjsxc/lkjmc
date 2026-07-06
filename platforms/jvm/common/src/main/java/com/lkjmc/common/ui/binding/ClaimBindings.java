package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

public final class ClaimBindings {
    private ClaimBindings() {}

    public static List<MenuBinding> bindings() {
        return List.of(new Claims(), new ClaimDetail());
    }

    record Claim(String id, String ownerUuid, String ownerName, String name, long chunks) {}

    static List<Claim> claims(JsonObject body, String binding) {
        var values = new ArrayList<Claim>();
        for (var value : Jsons.array(body, "claims", binding)) {
            var row = Jsons.elementObject(value, binding);
            values.add(new Claim(Jsons.string(row, "id", binding), Jsons.string(row, "ownerUuid", binding),
                Jsons.string(row, "ownerName", binding), Jsons.string(row, "name", binding),
                Jsons.integer(row, "chunkCount", binding)));
        }
        return values.stream().sorted(Comparator.comparing(Claim::name)).toList();
    }

    private static EntryView row(Claim claim) {
        return Views.entry("FILLED_MAP", Views.lit(claim.name()),
            List.of(Views.lit(claim.ownerName()), Views.lit(claim.chunks()), Views.key("menu.claims.detail.lore")),
            ItemRole.NAVIGATION, Views.open("claim-detail", Map.of("claimId", claim.id())));
    }

    private static final class Claims extends BasicBinding {
        Claims() { super("claims", "daemon", List.of("claim.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var entries = claims(body, id()).stream().map(ClaimBindings::row).toList();
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, Views.keys("menu.claims.info.lore")));
        }
    }

    private static final class ClaimDetail extends BasicBinding {
        ClaimDetail() { super("claim-detail", "daemon", List.of("claim.snapshot")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var claimId = ctx.param("claimId").orElse("");
            for (var value : Jsons.array(body, "chunks", id())) {
                var row = Jsons.elementObject(value, id());
                var idValue = Jsons.string(row, "claimId", id());
                var trusts = Jsons.array(row, "trusts", id());
                if (!claimId.isBlank() && !claimId.equals(idValue)) { continue; }
                var name = Jsons.string(row, "name", id());
                var owner = Jsons.string(row, "ownerName", id());
                Jsons.string(row, "ownerUuid", id());
                Jsons.string(row, "instanceId", id());
                Jsons.string(row, "worldName", id());
                Jsons.integer(row, "chunkX", id());
                Jsons.integer(row, "chunkZ", id());
                var slots = List.of(
                    Views.keyedSlot(20, "RED_WOOL", "menu.claims.delete", ItemRole.ACTION,
                        Views.open("claim-confirm", Map.of("claimId", idValue)), ctx.params(), "menu.claims.delete.lore"),
                    Views.keyedSlot(24, "PLAYER_HEAD", "menu.claims.trust", ItemRole.NAVIGATION,
                        Views.open("claim-trust-picker", Map.of("claimId", idValue)), ctx.params(), "menu.claims.trust.lore"));
                return Views.data(new RouteView.DetailView(slots,
                    List.of(Views.lit(name), Views.lit(owner), Views.lit(trusts.size()))));
            }
            return BindingResult.empty();
        }
    }
}
