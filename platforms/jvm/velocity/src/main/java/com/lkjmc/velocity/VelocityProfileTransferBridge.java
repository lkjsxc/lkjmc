package com.lkjmc.velocity;

import com.lkjmc.common.transfer.ProfileTransferMessages;
import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.connection.PluginMessageEvent;
import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.ServerConnection;
import com.velocitypowered.api.proxy.messages.MinecraftChannelIdentifier;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.TimeUnit;
import java.util.Map;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityProfileTransferBridge implements ProfileSaveBridge {
    private final MinecraftChannelIdentifier channel = MinecraftChannelIdentifier.from(ProfileTransferMessages.CHANNEL);
    private final ConcurrentHashMap<UUID, CompletableFuture<Boolean>> pending = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<UUID, UUID> tpa = new ConcurrentHashMap<>();
    private ProxyServer proxy;

    public void register(ProxyServer proxy, Object plugin) {
        this.proxy = proxy;
        proxy.getChannelRegistrar().register(channel);
        proxy.getEventManager().register(plugin, this);
    }

    @Override
    public CompletableFuture<Boolean> save(Player player) {
        var requestId = UUID.randomUUID();
        var future = new CompletableFuture<Boolean>();
        pending.put(requestId, future);
        var sent = player.getCurrentServer()
            .map(server -> server.sendPluginMessage(channel, ProfileTransferMessages.saveRequest(requestId)))
            .orElse(false);
        if (!sent) {
            pending.remove(requestId);
            return CompletableFuture.completedFuture(false);
        }
        return future.orTimeout(5, TimeUnit.SECONDS).handle((ok, error) -> {
            pending.remove(requestId);
            return error == null && Boolean.TRUE.equals(ok);
        });
    }

    public void transfer(Player player, String targetServer) {
        if (proxy == null) {
            return;
        }
        var target = proxy.getServer(targetServer);
        if (target.isEmpty()) {
            player.sendMessage(VelocityMessages.message("velocity.target.unavailable", NamedTextColor.RED));
            return;
        }
        save(player).thenAccept(saved -> {
            if (saved) {
                player.createConnectionRequest(target.get()).fireAndForget();
            }
        });
    }

    @Subscribe
    public void onPluginMessage(PluginMessageEvent event) {
        if (!event.getIdentifier().equals(channel)) {
            return;
        }
        saved(event);
        routeTransfer(event);
        requestTpa(event);
        acceptTpa(event);
    }

    private void saved(PluginMessageEvent event) {
        ProfileTransferMessages.parse("saved", event.getData()).ifPresent(requestId -> {
            event.setResult(PluginMessageEvent.ForwardResult.handled());
            var future = pending.remove(requestId);
            if (future != null) {
                future.complete(true);
            }
        });
    }

    private void routeTransfer(PluginMessageEvent event) {
        ProfileTransferMessages.parseText("transfer", event.getData()).ifPresent(server -> {
            event.setResult(PluginMessageEvent.ForwardResult.handled());
            if (event.getSource() instanceof ServerConnection connection) {
                transfer(connection.getPlayer(), server);
            }
        });
    }

    private void requestTpa(PluginMessageEvent event) {
        ProfileTransferMessages.parseText("tpa", event.getData()).ifPresent(targetName -> {
            event.setResult(PluginMessageEvent.ForwardResult.handled());
            if (!(event.getSource() instanceof ServerConnection connection) || proxy == null) {
                return;
            }
            var source = connection.getPlayer();
            var target = proxy.getPlayer(targetName);
            if (target.isEmpty()) {
                source.sendMessage(VelocityMessages.message("velocity.player.unavailable", NamedTextColor.RED));
                return;
            }
            tpa.put(target.get().getUniqueId(), source.getUniqueId());
            target.get().sendMessage(VelocityMessages.message(
                "velocity.teleport.request",
                NamedTextColor.YELLOW,
                Map.of("player", source.getUsername())
            ));
        });
    }

    private void acceptTpa(PluginMessageEvent event) {
        ProfileTransferMessages.parseText("tpaccept", event.getData()).ifPresent(payload -> {
            event.setResult(PluginMessageEvent.ForwardResult.handled());
            if (!(event.getSource() instanceof ServerConnection connection) || proxy == null) {
                return;
            }
            var parts = payload.split("\\|", 2);
            if (parts.length != 2) {
                return;
            }
            var source = proxy.getPlayer(parts[0]);
            if (source.isEmpty() || !source.get().getUniqueId().equals(tpa.remove(connection.getPlayer().getUniqueId()))) {
                connection.getPlayer().sendMessage(VelocityMessages.message("velocity.teleport.none", NamedTextColor.RED));
                return;
            }
            completeTpa(source.get(), connection, parts[1]);
        });
    }

    private void completeTpa(Player source, ServerConnection targetConnection, String location) {
        save(source).thenAccept(saved -> {
            if (!saved) {
                source.sendMessage(VelocityMessages.message("velocity.source-save.timeout", NamedTextColor.RED));
                return;
            }
            source.createConnectionRequest(targetConnection.getServer()).connect().thenAccept(result ->
                source.sendPluginMessage(channel, ProfileTransferMessages.arrive(location)));
        });
    }
}
