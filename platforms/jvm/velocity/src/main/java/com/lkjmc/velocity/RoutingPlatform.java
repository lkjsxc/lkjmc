package com.lkjmc.velocity;

import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletionStage;

public interface RoutingPlatform {
    Set<String> registrations();
    Optional<RoutingTarget> route(String ownedId);
    boolean register(String ownedId, RoutingTarget route);
    boolean unregister(String ownedId);
    CompletionStage<Boolean> connect(UUID playerId, String ownedId);
}
