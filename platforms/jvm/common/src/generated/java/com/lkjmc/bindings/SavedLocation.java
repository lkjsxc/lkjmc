package com.lkjmc.bindings;

public record SavedLocation(
        String name,
        String server,
        String world,
        double x,
        double y,
        double z,
        double yaw,
        double pitch
) {
    public SavedLocation {
        java.util.Objects.requireNonNull(name, "name");
        if (name.isBlank()) throw new IllegalArgumentException("name");
        java.util.Objects.requireNonNull(server, "server");
        if (server.isBlank()) throw new IllegalArgumentException("server");
        java.util.Objects.requireNonNull(world, "world");
        if (world.isBlank()) throw new IllegalArgumentException("world");
    }
}
