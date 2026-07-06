package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

public final class SocialBindings {
    private SocialBindings() {}

    public static List<MenuBinding> bindings() {
        return List.of(new Mail(), new Party(), new Reports(), new ReportDetail());
    }

    record Report(String id, String reporter, String target, String server, String reason, String status) {}

    static List<Report> reports(JsonObject body, String binding) {
        var values = new ArrayList<Report>();
        for (var value : Jsons.array(body, "reports", binding)) {
            var row = Jsons.elementObject(value, binding);
            values.add(new Report(Jsons.string(row, "id", binding), Jsons.string(row, "reporterUuid", binding),
                Jsons.string(row, "targetUuid", binding), Jsons.string(row, "serverId", binding),
                Jsons.string(row, "reason", binding), Jsons.string(row, "status", binding)));
        }
        return values.stream().sorted(Comparator.comparing(Report::id)).toList();
    }

    private static EntryView reportRow(Report report) {
        return Views.entry("REDSTONE_TORCH", Views.lit(report.id()),
            List.of(Views.lit(report.server()), Views.lit(report.status()), Views.lit(snippet(report.reason())),
                Views.key("menu.reports.detail.lore")), ItemRole.NAVIGATION,
            Views.open("report-detail", Map.of("reportId", report.id())));
    }

    private static String snippet(String text) {
        var value = text == null ? "" : text;
        return value.length() <= 48 ? value : value.substring(0, 45) + "...";
    }

    private static final class Mail extends BasicBinding {
        Mail() { super("mail", "daemon", List.of("player.mail.inbox")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var entries = new ArrayList<EntryView>();
            for (var value : Jsons.array(body, "messages", id())) {
                var row = Jsons.elementObject(value, id());
                var message = Jsons.string(row, "id", id());
                var read = Jsons.bool(row, "read", id());
                entries.add(Views.entry(read ? "BOOK" : "WRITABLE_BOOK",
                    Views.lit(Jsons.string(row, "senderName", id())),
                    List.of(Views.lit(snippet(Jsons.string(row, "body", id()))), Views.key("menu.mail.read.lore")),
                    ItemRole.ACTION, Views.command("mail read " + message)));
            }
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, Views.keys("menu.mail.info.lore")));
        }
    }

    private static final class Party extends BasicBinding {
        Party() { super("party", "daemon", List.of("player.party.info")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var found = Jsons.bool(body, "found", id());
            var slots = new ArrayList<com.lkjmc.common.ui.kernel.FrameSlot>();
            if (found) {
                slots.add(Views.slot(20, "NAME_TAG", Views.lit(Jsons.string(body, "name", id())),
                    List.of(Views.lit(Jsons.string(body, "role", id()))), ItemRole.INFO,
                    new com.lkjmc.common.ui.document.DocumentAction.None(), ctx.params()));
            } else {
                slots.add(Views.keyedSlot(20, "BARRIER", "menu.party.none", ItemRole.INFO,
                    new com.lkjmc.common.ui.document.DocumentAction.None(), ctx.params(), "menu.party.none.lore"));
            }
            slots.add(Views.keyedSlot(22, "LIME_DYE", "menu.party.create", ItemRole.ACTION,
                Views.daemon("player.party.create", Map.of("playerUuid", ctx.playerUuid()),
                    "party.created", "party.failed", true), ctx.params(), "menu.party.create.lore"));
            var invite = found ? Views.open("party-invite-picker") : Views.disabled("menu.disabled.no-party");
            slots.add(Views.keyedSlot(24, "PAPER", "menu.party.invite", found ? ItemRole.NAVIGATION : ItemRole.DISABLED,
                invite, ctx.params(), "menu.party.invite.lore"));
            var leave = found ? Views.open("party-confirm") : Views.disabled("menu.disabled.no-party");
            slots.add(Views.keyedSlot(31, "RED_DYE", "menu.party.leave", found ? ItemRole.ACTION : ItemRole.DISABLED,
                leave, ctx.params(), "menu.party.leave.lore"));
            return Views.data(new RouteView.CustomView("party", slots, Views.keys("menu.party.info.lore")));
        }
    }

    private static final class Reports extends BasicBinding {
        Reports() { super("reports", "daemon", List.of("player.report.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            if (!ctx.permissions().reports()) { return BindingResult.denied(); }
            var entries = reports(body, id()).stream().map(SocialBindings::reportRow).toList();
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, Views.keys("menu.reports.info.lore")));
        }
    }

    private static final class ReportDetail extends BasicBinding {
        ReportDetail() { super("report-detail", "daemon", List.of("player.report.list")); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            if (!ctx.permissions().reports()) { return BindingResult.denied(); }
            var reportId = ctx.param("reportId").orElse("");
            return reports(body, id()).stream().filter(r -> r.id().equals(reportId)).findFirst()
                .<BindingResult>map(this::detail).orElseGet(BindingResult::empty);
        }
        private BindingResult detail(Report report) {
            var slots = List.of(
                Views.keyedSlot(20, "LIME_WOOL", "menu.reports.resolve", ItemRole.ACTION,
                    Views.open("report-confirm", Map.of("reportId", report.id())), Map.of(), "menu.reports.confirm.lore"),
                Views.keyedSlot(24, "RED_WOOL", "menu.reports.dismiss", ItemRole.ACTION,
                    Views.daemon("player.report.dismiss", Map.of("reportId", report.id()),
                        "reports.closed", "reports.close.failed", true), Map.of(), "menu.reports.confirm.lore"));
            return Views.data(new RouteView.DetailView(slots,
                List.of(Views.lit(report.server()), Views.lit(report.status()), Views.lit(snippet(report.reason())))));
        }
    }
}
