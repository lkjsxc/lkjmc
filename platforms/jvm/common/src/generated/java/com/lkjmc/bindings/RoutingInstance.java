package com.lkjmc.bindings;

import java.util.List;

public record RoutingInstance(
        String id,
        String kind,
        String desiredState,
        String observedState,
        Boolean healthy,
        Boolean ready,
        Integer playerCount,
        List<RoutingPort> ports
) {
    public RoutingInstance {
        java.util.Objects.requireNonNull(id, "id");
        if (id.isBlank()) throw new IllegalArgumentException("id");
        java.util.Objects.requireNonNull(kind, "kind");
        if (kind.isBlank()) throw new IllegalArgumentException("kind");
        java.util.Objects.requireNonNull(desiredState, "desiredState");
        if (desiredState.isBlank()) throw new IllegalArgumentException("desiredState");
        ports = List.copyOf(ports);
    }
}
