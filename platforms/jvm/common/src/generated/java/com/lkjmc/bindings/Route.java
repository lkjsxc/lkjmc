package com.lkjmc.bindings;


public record Route(
        String host,
        String id,
        int port,
        boolean ready
) {
    public Route {
        java.util.Objects.requireNonNull(host, "host");
        if (host.isBlank()) throw new IllegalArgumentException("host");
        java.util.Objects.requireNonNull(id, "id");
        if (id.isBlank()) throw new IllegalArgumentException("id");
    }
}
