package com.lkjmc.paper;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.permission.PermissionNodes;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.ArrayList;
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
    private final EndExpeditionReturnService returnService;

    public EndExpeditionCommandAdapter(
        LkjmcPaperPlugin plugin,
        MessageRenderer renderer,
        EndExpeditionReturnService returnService
    ) {
        this.plugin = plugin;
        this.renderer = renderer;
        this.returnService = returnService;
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
            return returnService.returnToHub(player);
        }
        var includeParty = includeParty(player, args);
        if (includeParty.isEmpty()) {
            return true;
        }
        send(player, purchaseCommand(), purchaseBody(player, includeParty.get()),
            response -> handlePurchase(player, response.ok(), response.body()));
        return true;
    }

    static String purchaseCommand() {
        return "adventure.end.purchase";
    }

    static Map<String, Object> purchaseBody(Player player, boolean includeParty) {
        return Map.of(
            "playerUuid", player.getUniqueId().toString(),
            "playerName", player.getName(),
            "includeParty", includeParty
        );
    }

    private Optional<Boolean> includeParty(Player player, String[] args) {
        if (args.length == 0) {
            return Optional.of(false);
        }
        if (args.length == 1 && args[0].equalsIgnoreCase("party")) {
            return Optional.of(true);
        }
        player.sendMessage(renderer.render(plugin.localeService().locale(player),
            "command.usage", Map.of("usage", "/endexpedition [party|return]")));
        return Optional.empty();
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
        var participants = new ArrayList<UUID>();
        for (JsonElement item : body.getAsJsonArray("participants")) {
            if (item.isJsonObject()) {
                DaemonJson.string(item.getAsJsonObject(), "playerUuid").flatMap(this::parseUuid).ifPresent(participants::add);
            }
        }
        plugin.scheduler().runGlobal(() -> participants.stream().map(plugin.getServer()::getPlayer)
            .filter(java.util.Objects::nonNull)
            .forEach(player -> plugin.scheduler().runPlayer(player, () -> requestIntent(player, target))));
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
        return renderer.render(plugin.localeService().locale(player), key, Map.of());
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
