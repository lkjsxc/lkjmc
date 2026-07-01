package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonJson;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.daemon.DaemonResponse;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.time.Duration;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicBoolean;
import org.bukkit.entity.Player;

public final class EndExpeditionReturnService {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final AtomicBoolean expiryReturnStarted = new AtomicBoolean(false);

    public EndExpeditionReturnService(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
    }

    public void startExpiryWatcher() {
        if (!instanceId().startsWith("end-") || plugin.daemon().isEmpty()) {
            return;
        }
        plugin.scheduler().runAsyncRepeating(this::pollExpiry,
            Duration.ofSeconds(15), Duration.ofSeconds(15));
    }

    public boolean returnToHub(Player player) {
        var body = Map.<String, Object>of(
            "playerUuid", player.getUniqueId().toString(),
            "playerName", player.getName(),
            "temporaryInstanceId", instanceId()
        );
        send(player, "adventure.end.return", body, response -> handleReturn(player, response.ok(), response.body()));
        return true;
    }

    private void pollExpiry() {
        plugin.daemon().ifPresent(client -> client.send(request("temporary.instance.get",
            Map.of("id", instanceId()))).thenAccept(this::handleExpiry));
    }

    private void handleExpiry(DaemonResponse response) {
        if (!response.ok() || !"adventure".equals(DaemonJson.string(response.body(), "ownerKind").orElse(""))) {
            return;
        }
        var expires = DaemonJson.integer(response.body(), "expiresInSeconds").orElse(Long.MAX_VALUE);
        if (expires > 30 || !expiryReturnStarted.compareAndSet(false, true)) {
            return;
        }
        for (Player player : plugin.getServer().getOnlinePlayers()) {
            plugin.scheduler().runPlayer(player, () -> returnToHub(player));
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

    private void send(Player player, String command, Map<String, Object> body,
                      java.util.function.Consumer<DaemonResponse> handler) {
        plugin.daemon().ifPresentOrElse(client -> client.send(request(command, body))
            .thenAccept(response -> plugin.scheduler().runPlayer(player, () -> handler.accept(response))),
            () -> player.sendMessage(message(player, "daemon.unavailable")));
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
