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
        plugin.daemon().ifPresentOrElse(client -> client.send(request("adventure.end.purchase", body))
            .thenAccept(response -> plugin.scheduler().runPlayer(player,
                () -> handle(player, response.ok(), response.body()))),
            () -> player.sendMessage(message(player, "daemon.unavailable")));
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
            "command.usage", Map.of("usage", "/endexpedition [party]")));
        return Optional.empty();
    }

    private void handle(Player player, boolean ok, JsonObject body) {
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

    private void requestIntent(Player player, String target) {
        var body = Map.<String, Object>of(
            "playerUuid", player.getUniqueId().toString(),
            "playerName", player.getName(),
            "temporaryInstanceId", target
        );
        plugin.daemon().ifPresent(client -> client.send(request("temporary.transfer.intent", body))
            .thenAccept(response -> plugin.scheduler().runPlayer(player, () -> {
                if (response.ok()) {
                    player.sendPluginMessage(plugin, ProfileTransferMessages.CHANNEL,
                        ProfileTransferMessages.transferRequest(target));
                    player.sendMessage(message(player, "adventure.end.started"));
                } else {
                    player.sendMessage(message(player, "adventure.end.failed"));
                }
            })));
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
