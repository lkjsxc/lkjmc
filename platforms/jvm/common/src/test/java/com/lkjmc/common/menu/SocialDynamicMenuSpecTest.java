package com.lkjmc.common.menu;

import static com.lkjmc.common.menu.MenuSpecAssertions.actionAt;
import static com.lkjmc.common.menu.MenuSpecAssertions.assertSlot;
import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

final class SocialDynamicMenuSpecTest {
    @Test
    void partyStatusUsesDaemonDataAndConfirmsLeave() {
        var spec = PartyDynamicMenus.party(new PartyStatus(true, "Raiders", "owner", true));
        assertSlot(spec, 20, "literal:Raiders");
        assertEquals(new MenuAction.TextInput("menu.input.party-name.prompt", "party create"), actionAt(spec, 22));
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("party-invite-picker"))), actionAt(spec, 24));
        assertEquals(new MenuAction.OpenRoute(new MenuRoute(new MenuId("party-confirm"))), actionAt(spec, 31));
        assertEquals(new MenuAction.RunPlayerCommand("party leave"), actionAt(PartyDynamicMenus.partyConfirm(), 11));
    }

    @Test
    void reportListUsesDaemonDataAndOpensDetail() {
        var spec = ReportDynamicMenus.reports(List.of(new ReportMenuEntry("12345678-aaaa", "hub", "grief", "open")));
        assertSlot(spec, 19, "literal:report 12345678");
        var route = new MenuRoute(new MenuId("report-detail"), Map.of(
            "reportId", "12345678-aaaa", "serverId", "hub", "reason", "grief", "status", "open"));
        assertEquals(new MenuAction.OpenRoute(route), actionAt(spec, 19));
    }

    @Test
    void reportDetailConfirmsResolveAndDismiss() {
        var spec = ReportDynamicMenus.reportDetail(new ReportMenuEntry("12345678-aaaa", "hub", "grief", "open"));
        assertEquals(new MenuAction.OpenRoute(confirm("resolve", "12345678-aaaa")), actionAt(spec, 20));
        assertEquals(new MenuAction.OpenRoute(confirm("dismiss", "12345678-aaaa")), actionAt(spec, 24));
        var confirm = ReportDynamicMenus.reportConfirm("resolve", "12345678-aaaa");
        assertEquals(new MenuAction.RunPlayerCommand("reports resolve 12345678-aaaa"), actionAt(confirm, 11));
    }

    @Test
    void reportListCanRenderPermissionDeniedWithoutDaemonData() {
        var spec = ReportDynamicMenus.reports(List.of(), false);
        assertSlot(spec, 22, "menu.reports.denied");
        assertEquals(new MenuAction.Disabled("menu.disabled.reports-permission"), actionAt(spec, 22));
    }

    @Test
    void mailInboxUsesDaemonDataAndReadCommands() {
        var id = "00000000-0000-0000-0000-000000000000";
        var spec = MailDynamicMenus.mail(List.of(new MailMenuEntry(id, "Alex", "hi", false)));
        assertSlot(spec, 19, "literal:Alex");
        assertEquals(new MenuAction.RunPlayerCommand("mail read " + id), actionAt(spec, 19));
    }

    private static MenuRoute confirm(String action, String reportId) {
        return new MenuRoute(new MenuId("report-confirm"), Map.of("reportId", reportId, "action", action));
    }
}
