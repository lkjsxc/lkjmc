package com.lkjmc.velocity;

public final class VelocityTransferCoordinator {
    public boolean canTransfer(String sourceServer, String targetServer) {
        return sourceServer != null && targetServer != null && !sourceServer.equals(targetServer);
    }
}
