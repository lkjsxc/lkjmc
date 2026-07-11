package com.lkjmc.velocity;

import com.velocitypowered.api.proxy.Player;
import com.velocitypowered.api.proxy.server.RegisteredServer;
import java.util.concurrent.CompletableFuture;

public final class VelocityTransferCoordinator {
    public boolean canTransfer(String sourceServer, String targetServer) {
        return sourceServer != null && targetServer != null && !sourceServer.equals(targetServer);
    }

    public CompletableFuture<Boolean> connect(Player player, RegisteredServer target) {
        return player.createConnectionRequest(target).connect().thenApply(result -> result.isSuccessful());
    }
}
