package com.lkjmc.smoke;

import java.time.Duration;

final class EconomySmoke {
    private static final Duration SHORT = Duration.ofSeconds(10);
    private static final Duration LONG = Duration.ofSeconds(20);

    private EconomySmoke() {}

    static void run(SmokeClient client) throws InterruptedException {
        daily(client);
        shopPurchase(client);
        Thread.sleep(1000L);
        client.command("exchange cobblestone all");
        client.awaitMessage("Exchanged 64 COBBLESTONE for 64 points", LONG);
        openEconomy(client, 21, "Kits", "No kits configured");
        openEconomy(client, 23, "Votes", "No vote links");
    }

    private static void daily(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(22);
        client.awaitTitle("Economy", SHORT);
        client.click(22);
        client.awaitTitle("Daily reward", LONG);
        client.awaitItem("Claim daily", LONG);
        client.click(22);
        client.awaitMessage("Daily reward claimed.", LONG);
        client.assertStillOpen("Daily reward", Duration.ofSeconds(3));
    }

    private static void shopPurchase(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(22);
        client.awaitTitle("Economy", SHORT);
        client.click(20);
        client.awaitTitle("Shop", LONG);
        client.clickItem("Cobblestone x64", LONG);
        client.awaitMessage("Purchase complete.", LONG);
    }

    private static void openEconomy(SmokeClient client, int slot, String title, String item) throws InterruptedException {
        root(client);
        client.click(22);
        client.awaitTitle("Economy", SHORT);
        client.click(slot);
        client.awaitTitle(title, LONG);
        client.awaitItem(item, LONG);
    }

    private static void root(SmokeClient client) throws InterruptedException {
        client.command("menu");
        client.awaitTitle("lkjmc Menu", LONG);
    }
}
