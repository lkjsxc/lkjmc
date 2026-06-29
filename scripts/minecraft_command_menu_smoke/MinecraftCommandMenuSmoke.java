package com.lkjmc.smoke;

import java.time.Duration;

public final class MinecraftCommandMenuSmoke {
    private static final Duration SHORT = Duration.ofSeconds(10);
    private static final Duration LONG = Duration.ofSeconds(20);

    private MinecraftCommandMenuSmoke() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("usage: <host> <port>");
        }
        try (var client = new SmokeClient(args[0], Integer.parseInt(args[1]))) {
            client.connect();
            commands(client);
            serverList(client);
            profile(client);
            claims(client);
            mailAndReports(client);
            settings(client);
            client.assertNoParserLeak();
        }
        System.out.println("ok minecraft command menu smoke");
    }

    private static void commands(SmokeClient client) throws InterruptedException {
        client.assertSuggestions("/lkjmc ", "status", "doctor", "server");
        client.assertSuggestions("/lkjmc server ", "list", "start", "stop", "restart", "create", "delete");
        client.command("lkjmc status");
        client.awaitMessage("lkjmc velocity running", LONG);
        client.awaitMessage("ok status", LONG);
        client.command("lkjmc doctor");
        client.awaitMessage("lkjmc doctor", LONG);
        client.awaitMessage("ok doctor", LONG);
        client.command("lkjmc server");
        client.awaitMessage("/lkjmc server list|start|stop|restart|create|delete", LONG);
        client.command("lkjmc server list");
        client.awaitMessage("servers:", LONG);
        client.assertNoParserLeak();
    }

    private static void serverList(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(19);
        client.awaitTitle("Network & Servers", SHORT);
        client.click(20);
        client.awaitTitle("Servers", LONG);
        client.awaitItem("hub", LONG);
        client.assertStillOpen("Servers", Duration.ofSeconds(3));
    }

    private static void profile(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(24);
        client.awaitTitle("Profile", LONG);
        client.awaitItem("Point balance", LONG);
        client.assertStillOpen("Profile", Duration.ofSeconds(3));
    }

    private static void claims(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(21);
        client.awaitTitle("Claims", LONG);
        client.awaitItem("No claims yet", LONG);
        client.click(22);
        client.assertStillOpen("Claims", Duration.ofSeconds(3));
    }

    private static void mailAndReports(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(23);
        client.awaitTitle("Social", SHORT);
        client.click(20);
        client.awaitTitle("Mail", LONG);
        client.awaitItem("No mail", LONG);
        client.click(22);
        client.assertStillOpen("Mail", Duration.ofSeconds(3));
        root(client);
        client.click(23);
        client.awaitTitle("Social", SHORT);
        client.click(24);
        client.awaitTitle("Reports", LONG);
        client.awaitItem("No open reports", LONG);
        client.click(22);
        client.assertStillOpen("Reports", Duration.ofSeconds(3));
    }

    private static void settings(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(25);
        client.awaitTitle("Settings", SHORT);
        client.awaitItem("Toggle HUD", SHORT);
        client.click(22);
        client.awaitMessage("HUD ", LONG);
        client.assertStillOpen("Settings", Duration.ofSeconds(3));
        client.click(24);
        client.awaitMessage("Hotbar menu token", LONG);
        client.assertStillOpen("Settings", Duration.ofSeconds(3));
    }

    private static void root(SmokeClient client) throws InterruptedException {
        client.command("menu");
        client.awaitTitle("lkjmc Menu", LONG);
    }
}
