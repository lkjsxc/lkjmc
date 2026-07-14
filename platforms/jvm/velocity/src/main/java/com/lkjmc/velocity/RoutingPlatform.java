package com.lkjmc.velocity;

import com.lkjmc.bindings.Route;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletionStage;

public interface RoutingPlatform {
    Set<String> registrations();
    Optional<Route> route(String ownedId);
    boolean register(String ownedId, Route route);
    boolean unregister(String ownedId);
    CompletionStage<Boolean> connect(UUID playerId, String ownedId);
}
