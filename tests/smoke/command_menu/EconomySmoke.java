package com.lkjmc.smoke;

import java.time.Duration;
import java.util.Map;

final class EconomySmoke {
    private static final Duration SHORT = Duration.ofSeconds(10);
    private static final Duration LONG = Duration.ofSeconds(20);
    private static SmokeText text;

    private EconomySmoke() {}

    static void run(SmokeClient client, SmokeText locale) throws InterruptedException {
        text = locale;
        daily(client);
        shopPurchase(client);
        Thread.sleep(1000L);
        client.command("exchange cobblestone all");
        client.awaitMessage(text.format("exchange.ok", Map.of(
            "amount", "64", "material", "COBBLESTONE", "points", "64")), LONG);
        openEconomy(client, 20, text.key("menu.kits.title"), text.key("menu.kits.empty"));
        openEconomy(client, 22, text.key("menu.votes.title"), text.key("menu.votes.empty"));
    }

    private static void daily(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(22);
        client.awaitTitle(text.key("menu.economy.title"), SHORT);
        client.click(21);
        client.awaitTitle(text.key("menu.daily.title"), LONG);
        client.awaitItem(text.key("menu.daily.claim"), LONG);
        client.clickItem(text.key("menu.daily.claim"), LONG);
        client.awaitMessage(text.key("daily.claimed"), LONG);
        client.assertStillOpen(text.key("menu.daily.title"), Duration.ofSeconds(3));
    }

    private static void shopPurchase(SmokeClient client) throws InterruptedException {
        root(client);
        client.click(22);
        client.awaitTitle(text.key("menu.economy.title"), SHORT);
        client.click(19);
        client.awaitTitle(text.key("menu.shop.title"), LONG);
        client.click(11);
        client.awaitTitle(text.key("menu.shop.title"), LONG);
        client.click(49);
        client.awaitTitle(text.key("menu.economy.title"), LONG);
        client.click(19);
        client.awaitTitle(text.key("menu.shop.title"), LONG);
        client.clickItem(text.key("shop.item.building-cobblestone-64"), LONG);
        client.awaitMessage(text.key("shop.purchase.ok"), LONG);
    }

    private static void openEconomy(SmokeClient client, int slot, String title, String item) throws InterruptedException {
        root(client);
        client.click(22);
        client.awaitTitle(text.key("menu.economy.title"), SHORT);
        client.click(slot);
        client.awaitTitle(title, LONG);
        client.awaitItem(item, LONG);
    }

    private static void root(SmokeClient client) throws InterruptedException {
        client.command("menu");
        client.awaitTitle(text.key("menu.root.title"), LONG);
    }
}
