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

public final class VelocityProfileTransferBridge {
    private final MinecraftChannelIdentifier channel = MinecraftChannelIdentifier.from(ProfileTransferMessages.CHANNEL);
    private final ConcurrentHashMap<UUID, CompletableFuture<Boolean>> pending = new ConcurrentHashMap<>();
    private ProxyServer proxy;

    public void register(ProxyServer proxy, Object plugin) {
        this.proxy = proxy;
        proxy.getChannelRegistrar().register(channel);
        proxy.getEventManager().register(plugin, this);
    }

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
            player.sendMessage(net.kyori.adventure.text.Component.text("target unavailable"));
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
        ProfileTransferMessages.parse("saved", event.getData()).ifPresent(requestId -> {
            event.setResult(PluginMessageEvent.ForwardResult.handled());
            var future = pending.remove(requestId);
            if (future != null) {
                future.complete(true);
            }
        });
        ProfileTransferMessages.parseText("transfer", event.getData()).ifPresent(server -> {
            event.setResult(PluginMessageEvent.ForwardResult.handled());
            if (event.getSource() instanceof ServerConnection connection) {
                transfer(connection.getPlayer(), server);
            }
        });
    }
}
