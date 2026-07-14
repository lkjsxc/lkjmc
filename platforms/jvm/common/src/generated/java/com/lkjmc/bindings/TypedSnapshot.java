package com.lkjmc.bindings;

public sealed interface TypedSnapshot extends SyncResponse permits PermissionSnapshot, ClaimSnapshot, MenuSnapshot, ProfileSnapshot, PresenceSnapshot, RoutingSnapshot, SettingsSnapshot {
    String domain(); String key(); long revision();
    java.time.Instant generatedAt(); long credentialRevision(); DomainPayload payload();
}
