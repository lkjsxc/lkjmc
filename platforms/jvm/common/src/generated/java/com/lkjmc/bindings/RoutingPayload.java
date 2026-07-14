package com.lkjmc.bindings;

import java.util.List;

public record RoutingPayload(
        List<RoutingInstance> instances
) implements DomainPayload {
    public RoutingPayload {
        instances = List.copyOf(instances);
    }
}
