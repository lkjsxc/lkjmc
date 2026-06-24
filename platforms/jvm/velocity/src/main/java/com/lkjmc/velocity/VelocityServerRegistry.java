package com.lkjmc.velocity;

import java.util.List;

public record VelocityServerRegistry(List<String> registeredServers) {
    public VelocityServerRegistry {
        registeredServers = List.copyOf(registeredServers == null ? List.of() : registeredServers);
    }
}
