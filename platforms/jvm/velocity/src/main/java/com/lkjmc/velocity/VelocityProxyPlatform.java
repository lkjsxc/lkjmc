package com.lkjmc.velocity;

import com.lkjmc.bindings.Route;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.proxy.server.ServerInfo;
import java.net.InetSocketAddress;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import java.util.stream.Collectors;

public final class VelocityProxyPlatform implements RoutingPlatform {
    private final ProxyServer proxy;

    public VelocityProxyPlatform(ProxyServer proxy) {
        this.proxy = proxy;
    }

    @Override
    public Set<String> registrations() {
        return proxy.getAllServers().stream().map(server -> server.getServerInfo().getName())
                .collect(Collectors.toUnmodifiableSet());
    }

    @Override
    public Optional<Route> route(String ownedId) {
        return proxy.getServer(ownedId).map(server -> {
            var address = server.getServerInfo().getAddress();
            String id = ownedId.startsWith(VelocityRoutingAdapter.OWNED_PREFIX)
                    ? ownedId.substring(VelocityRoutingAdapter.OWNED_PREFIX.length()) : ownedId;
            return new Route(address.getHostString(), id, address.getPort(), true);
        });
    }

    @Override
    public boolean register(String ownedId, Route route) {
        if (proxy.getServer(ownedId).isPresent()) return true;
        ServerInfo info = new ServerInfo(ownedId, new InetSocketAddress(route.host(), route.port()));
        proxy.registerServer(info);
        return proxy.getServer(ownedId).isPresent();
    }

    @Override
    public boolean unregister(String ownedId) {
        return proxy.getServer(ownedId).map(server -> {
            proxy.unregisterServer(server.getServerInfo());
            return proxy.getServer(ownedId).isEmpty();
        }).orElse(true);
    }

    @Override
    public CompletionStage<Boolean> connect(UUID playerId, String ownedId) {
        var player = proxy.getPlayer(playerId);
        var server = proxy.getServer(ownedId);
        if (player.isEmpty() || server.isEmpty()) return CompletableFuture.completedFuture(false);
        return player.get().createConnectionRequest(server.get()).connect()
                .thenApply(result -> result.isSuccessful());
    }
}
