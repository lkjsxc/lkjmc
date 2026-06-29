package com.lkjmc.smoke;

import java.time.Duration;

public final class MinecraftCommandMenuSmoke {
    private MinecraftCommandMenuSmoke() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("usage: <host> <port>");
        }
        try (var client = new SmokeClient(args[0], Integer.parseInt(args[1]))) {
            client.connect();
            client.assertSuggestions("/lkjmc ", "status", "doctor", "server");
            client.assertSuggestions("/lkjmc server ", "list", "start", "stop", "restart", "create", "delete");
            client.command("lkjmc status");
            client.awaitMessage("lkjmc velocity running", Duration.ofSeconds(20));
            client.awaitMessage("ok status", Duration.ofSeconds(20));
            client.command("lkjmc doctor");
            client.awaitMessage("lkjmc doctor", Duration.ofSeconds(20));
            client.awaitMessage("ok doctor", Duration.ofSeconds(20));
            client.command("lkjmc server");
            client.awaitMessage("/lkjmc server list|start|stop|restart|create|delete", Duration.ofSeconds(20));
            client.command("lkjmc server list");
            client.awaitMessage("servers:", Duration.ofSeconds(20));
            client.assertNoParserLeak();
            client.command("menu");
            client.awaitTitle("lkjmc Menu", Duration.ofSeconds(20));
            client.click(19);
            client.awaitTitle("Network & Servers", Duration.ofSeconds(10));
            client.click(20);
            client.awaitTitle("Servers", Duration.ofSeconds(20));
            client.awaitItem("hub", Duration.ofSeconds(20));
            client.assertStillOpen("Servers", Duration.ofSeconds(3));
            client.command("menu");
            client.awaitTitle("lkjmc Menu", Duration.ofSeconds(10));
            client.click(24);
            client.awaitTitle("Profile", Duration.ofSeconds(20));
            client.awaitItem("Point balance", Duration.ofSeconds(20));
            client.assertStillOpen("Profile", Duration.ofSeconds(3));
            client.assertNoParserLeak();
        }
        System.out.println("ok minecraft command menu smoke");
    }
}
