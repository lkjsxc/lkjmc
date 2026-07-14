package com.lkjmc.velocity;

public record RoutingTarget(String host, String id, int port) {
    public RoutingTarget {
        if (host == null || host.isBlank() || id == null || id.isBlank() || port < 1 || port > 65535) {
            throw new IllegalArgumentException("invalid routing target");
        }
    }
}
