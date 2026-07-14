package com.lkjmc.bindings;

public record RoutingPort(
        int port,
        String purpose
) {
    public RoutingPort {
        java.util.Objects.requireNonNull(purpose, "purpose");
        if (purpose.isBlank()) throw new IllegalArgumentException("purpose");
    }
}
