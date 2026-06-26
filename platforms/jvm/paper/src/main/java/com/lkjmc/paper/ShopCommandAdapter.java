package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.Map;
import java.util.UUID;
import org.bukkit.entity.Player;

public final class ShopCommandAdapter {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public ShopCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public boolean list(Player player) {
        return send(player, "player.shop.list", Map.of(), "shop.list");
    }

    public boolean buy(Player player, String[] args) {
        if (args.length != 1) {
            player.sendMessage(message(player, "command.usage", Map.of("usage", "/buy <item>")));
            return true;
        }
        return send(player, "player.shop.purchase", Map.of(
            "playerUuid", player.getUniqueId().toString(), "name", player.getName(), "itemId", args[0]
        ), "shop.purchase");
    }

    private boolean send(Player player, String command, Map<String, Object> body, String kind) {
        plugin.daemon().ifPresentOrElse(client -> client.send(new DaemonRequest(
            UUID.randomUUID(), new DaemonActor("paper-plugin", instanceId()), command, body
        )).thenAccept(response -> plugin.scheduler().runPlayer(player,
            () -> player.sendMessage(result(player, kind, response.ok(), response.body())))),
            () -> player.sendMessage(message(player, "daemon.unavailable", Map.of())));
        return true;
    }

    private String result(Player player, String kind, boolean ok, JsonObject body) {
        if (kind.equals("shop.list")) {
            var count = DaemonJson.arraySize(body, "items");
            return message(player, "shop.list.count", Map.of("count", Integer.toString(count)));
        }
        return message(player, ok ? "shop.purchase.ok" : "shop.purchase.denied", Map.of());
    }

    private String message(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
