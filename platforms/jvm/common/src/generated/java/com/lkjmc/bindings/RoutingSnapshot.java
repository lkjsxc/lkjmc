package com.lkjmc.bindings;

import java.time.Instant;
import java.util.List;

public record RoutingSnapshot(
        Instant expiresAt,
        long revision,
        List<Route> routes
) {
    public RoutingSnapshot {
        java.util.Objects.requireNonNull(expiresAt, "expiresAt");
        if (revision <= 0) throw new IllegalArgumentException("revision");
        routes = List.copyOf(routes);
    }
}
