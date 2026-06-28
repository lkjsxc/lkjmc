package com.lkjmc.paper;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.permission.PermissionNodes;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;

public final class EndExpeditionCommandAdapter implements CommandExecutor {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;

    public EndExpeditionCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (!player.hasPermission(PermissionNodes.USER_ADVENTURE)) {
            player.sendMessage(message(player, "command.no-permission"));
            return true;
        }
        if (args.length == 1 && args[0].equalsIgnoreCase("return")) {
            return returnToHub(player);
        }
        var includeParty = includeParty(player, args);
        if (includeParty.isEmpty()) {
            return true;
        }
        var body = Map.<String, Object>of(
            "playerUuid", player.getUniqueId().toString(),
            "playerName", player.getName(),
            "acceptMinecraftEula", true,
            "includeParty", includeParty.get()
        );
        send(player, "adventure.end.purchase", body, response -> handlePurchase(player, response.ok(), response.body()));
        return true;
    }

    private Optional<Boolean> includeParty(Player player, String[] args) {
        if (args.length == 0) {
            return Optional.of(false);
        }
        if (args.length == 1 && args[0].equalsIgnoreCase("party")) {
            return Optional.of(true);
        }
        player.sendMessage(renderer.render(player.locale().toLanguageTag(),
            "command.usage", Map.of("usage", "/endexpedition [party|return]")));
        return Optional.empty();
    }

    private boolean returnToHub(Player player) {
        var body = Map.<String, Object>of(
            "playerUuid", player.getUniqueId().toString(),
            "playerName", player.getName(),
            "temporaryInstanceId", instanceId()
        );
        send(player, "adventure.end.return", body, response -> handleReturn(player, response.ok(), response.body()));
        return true;
    }

    private void handlePurchase(Player player, boolean ok, JsonObject body) {
        var target = DaemonJson.string(body, "targetServer").orElse("");
        if (!ok || target.isBlank()) {
            player.sendMessage(message(player, "adventure.end.failed"));
            return;
        }
        transferParticipants(body, target);
    }

    private void transferParticipants(JsonObject body, String target) {
        if (!body.has("participants") || !body.get("participants").isJsonArray()) {
            return;
        }
        for (JsonElement element : body.getAsJsonArray("participants")) {
            if (!element.isJsonObject()) {
                continue;
            }
            var uuid = DaemonJson.string(element.getAsJsonObject(), "playerUuid").flatMap(this::parseUuid);
            uuid.map(plugin.getServer()::getPlayer).ifPresent(player -> requestIntent(player, target));
        }
    }

    private void handleReturn(Player player, boolean ok, JsonObject body) {
        var target = DaemonJson.string(body, "targetServer").orElse("hub");
        if (!ok || target.isBlank()) {
            player.sendMessage(message(player, "adventure.end.return.failed"));
            return;
        }
        player.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
            ProfileTransferMessages.transferRequest(target));
        player.sendMessage(message(player, "adventure.end.returned"));
    }

    private void requestIntent(Player player, String target) {
        var body = Map.<String, Object>of(
            "playerUuid", player.getUniqueId().toString(),
            "playerName", player.getName(),
            "temporaryInstanceId", target
        );
        send(player, "temporary.transfer.intent", body, response -> {
            if (response.ok()) {
                player.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
                    ProfileTransferMessages.transferRequest(target));
                player.sendMessage(message(player, "adventure.end.started"));
            } else {
                player.sendMessage(message(player, "adventure.end.failed"));
            }
        });
    }

    private void send(Player player, String command, Map<String, Object> body,
                      java.util.function.Consumer<com.lkjmc.common.daemon.DaemonResponse> handler) {
        plugin.daemon().ifPresentOrElse(client -> client.send(request(command, body))
            .thenAccept(response -> plugin.scheduler().runPlayer(player, () -> handler.accept(response))),
            () -> player.sendMessage(message(player, "daemon.unavailable")));
    }

    private Optional<UUID> parseUuid(String value) {
        try {
            return Optional.of(UUID.fromString(value));
        } catch (IllegalArgumentException ignored) {
            return Optional.empty();
        }
    }

    private static DaemonRequest request(String command, Map<String, Object> body) {
        return new DaemonRequest(UUID.randomUUID(),
            new DaemonActor("paper-plugin", instanceId()), command, body);
    }

    private String message(Player player, String key) {
        return renderer.render(player.locale().toLanguageTag(), key, Map.of());
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
