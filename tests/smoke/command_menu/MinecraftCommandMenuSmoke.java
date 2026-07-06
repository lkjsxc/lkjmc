package com.lkjmc.smoke;

import java.time.Duration;

public final class MinecraftCommandMenuSmoke {
    private static final Duration SHORT = Duration.ofSeconds(10);
    private static final Duration LONG = Duration.ofSeconds(20);
    private static SmokeText text;

    private MinecraftCommandMenuSmoke() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 2) { throw new IllegalArgumentException("usage: <host> <port>"); }
        text = SmokeText.load();
        try (var client = new SmokeClient(args[0], Integer.parseInt(args[1]))) {
            client.connect(); commands(client); serverList(client); profile(client); claims(client);
            mailAndReports(client); settingsActions(client); docs(client); travel(client);
            EconomySmoke.run(client, text); party(client); achievements(client); language(client);
            client.assertNoParserLeak();
        }
        System.out.println("ok minecraft command menu smoke");
    }

    private static void commands(SmokeClient client) throws InterruptedException {
        client.assertSuggestions("/lkjmc ", "status", "doctor", "server");
        client.assertSuggestions("/lkjmc server ", "list", "start", "stop", "restart", "create", "delete");
        client.command("lkjmc status"); client.awaitMessage("lkjmc velocity running", LONG); client.awaitMessage("ok status", LONG);
        client.command("lkjmc doctor"); client.awaitMessage("lkjmc doctor", LONG); client.awaitMessage("ok doctor", LONG);
        client.command("lkjmc server"); client.awaitMessage("/lkjmc server list|start|stop|restart|create|delete", LONG);
        client.command("lkjmc server list"); client.awaitMessage("servers:", LONG);
        client.assertNoParserLeak();
    }

    private static void serverList(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(19); client.awaitTitle(t("menu.network.title"), SHORT);
        client.awaitSlot(45, t("menu.main-menu"), SHORT); client.awaitSlot(49, t("menu.back"), SHORT); client.awaitSlot(53, t("menu.close"), SHORT);
        client.click(20);
        client.awaitTitle(t("menu.server-list.title"), LONG);
        client.awaitSlot(45, t("menu.main-menu"), LONG); client.awaitSlot(46, t("menu.page.previous"), LONG);
        client.awaitSlot(47, t("menu.page.info"), LONG); client.awaitSlot(48, t("menu.page.next"), LONG);
        client.awaitSlot(49, t("menu.back"), LONG); client.awaitSlot(50, t("menu.refresh"), LONG); client.awaitSlot(53, t("menu.close"), LONG);
        var closes = client.closeCount();
        client.click(50);
        client.awaitTitle(t("menu.server-list.title"), LONG);
        client.assertNoCloseSince(closes);
    }

    private static void profile(SmokeClient client) throws InterruptedException {
        root(client); client.click(24); client.awaitTitle(t("menu.profile.title"), LONG);
        client.awaitItem(t("menu.profile.points"), LONG);
        client.assertStillOpen(t("menu.profile.title"), Duration.ofSeconds(3));
    }

    private static void claims(SmokeClient client) throws InterruptedException {
        root(client); client.click(21); client.awaitTitle(t("menu.claims.title"), LONG);
        client.awaitItem(t("menu.claims.empty"), LONG);
        client.click(28);
        client.assertStillOpen(t("menu.claims.title"), Duration.ofSeconds(3));
    }

    private static void mailAndReports(SmokeClient client) throws InterruptedException {
        root(client); client.click(23); client.awaitTitle(t("menu.social.title"), SHORT);
        client.click(20); client.awaitTitle(t("menu.mail.title"), LONG); client.awaitItem(t("menu.mail.empty"), LONG);
        client.click(28);
        client.assertStillOpen(t("menu.mail.title"), Duration.ofSeconds(3));
        root(client); client.click(23); client.awaitTitle(t("menu.social.title"), SHORT);
        client.click(24); client.awaitTitle(t("menu.reports.title"), LONG); client.awaitItem(t("menu.reports.empty"), LONG);
        client.click(28);
        client.assertStillOpen(t("menu.reports.title"), Duration.ofSeconds(3));
    }

    private static void settingsActions(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(25);
        client.awaitTitle(t("menu.settings.title"), SHORT);
        client.awaitSlot(50, t("menu.refresh"), LONG);
        client.awaitItem(t("menu.hud.toggle"), LONG);
        var hud = client.awaitSlotAny(22, LONG, t("hud.enabled"), t("hud.disabled"));
        var closes = client.closeCount();
        client.click(22);
        client.awaitSlot(22, flip(hud, t("hud.enabled"), t("hud.disabled")), LONG);
        client.assertNoCloseSince(closes);
        var token = client.awaitSlotAny(24, LONG, t("hotbar.menu.enabled"), t("hotbar.menu.disabled"));
        closes = client.closeCount();
        client.click(24);
        client.awaitSlot(24, flip(token, t("hotbar.menu.enabled"), t("hotbar.menu.disabled")), LONG);
        client.assertNoCloseSince(closes);
    }

    private static void docs(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(30);
        client.awaitTitle(t("menu.docs.directory.title"), LONG);
        client.awaitSlot(45, t("menu.main-menu"), LONG);
        client.awaitSlot(49, t("menu.parent"), LONG);
        client.awaitSlot(53, t("menu.close"), LONG);
        client.click(19);
        client.awaitTitle(t("menu.docs.directory.title"), LONG);
        client.click(49);
        client.awaitTitle(t("menu.docs.directory.title"), LONG);
        client.command("docs docs/product/gui/menu-framework.md");
        client.awaitTitle(t("menu.docs.file.title"), LONG);
        client.awaitSlot(21, t("docs.previous"), LONG);
        client.awaitSlot(23, t("docs.next"), LONG);
        client.awaitSlot(4, "1/", LONG);
        client.click(23);
        client.awaitSlot(4, "2/", LONG);
        client.click(21);
        client.awaitSlot(4, "1/", LONG);
        client.click(49);
        client.awaitTitle(t("menu.root.title"), LONG);
        client.command("docs docs/product/gui/menu-framework.md");
        client.awaitTitle(t("menu.docs.file.title"), LONG);
        var closes = client.closeCount();
        client.click(53);
        client.awaitCloseAfter(closes, LONG);
    }

    private static void travel(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(20);
        client.awaitTitle(t("menu.travel.title"), SHORT);
        client.click(19);
        client.awaitTitle(t("menu.homes.title"), LONG);
        client.awaitItem(t("menu.homes.empty"), LONG);
        client.click(28);
        client.assertStillOpen(t("menu.homes.title"), Duration.ofSeconds(3));
        root(client);
        client.click(20);
        client.awaitTitle(t("menu.travel.title"), SHORT);
        client.click(21);
        client.awaitTitle(t("menu.warps.title"), LONG);
        client.awaitItem(t("menu.warps.empty"), LONG);
    }

    private static void party(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(23);
        client.awaitTitle(t("menu.social.title"), SHORT);
        client.click(22);
        client.awaitTitle(t("menu.party.title"), LONG);
        client.awaitItem(t("menu.party.none"), LONG);
        client.click(24);
        client.assertStillOpen(t("menu.party.title"), Duration.ofSeconds(3));
    }

    private static void achievements(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(24);
        client.awaitTitle(t("menu.profile.title"), LONG);
        client.awaitItem(t("menu.profile.achievements"), LONG);
        client.click(22);
        client.awaitTitle(t("menu.achievements.title"), LONG);
        client.awaitItem(t("menu.achievements.info"), LONG);
    }

    private static void language(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(25);
        client.awaitTitle(t("menu.settings.title"), SHORT);
        client.click(20);
        client.awaitTitle(t("menu.language.title"), SHORT);
        client.awaitItem(t("language.english"), SHORT);
        client.click(20);
        client.awaitMessage(t("language.saved"), LONG);
        client.assertStillOpen(t("menu.language.title"), Duration.ofSeconds(3));
    }

    private static void root(SmokeClient client) throws InterruptedException {
        client.command("menu");
        client.awaitTitle(t("menu.root.title"), LONG);
        client.awaitSlot(53, t("menu.close"), LONG);
    }

    private static String flip(String value, String on, String off) { return value.equals(on) ? off : on; }
    private static String t(String key) { return text.key(key); }
}
