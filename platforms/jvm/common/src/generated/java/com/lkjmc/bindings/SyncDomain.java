package com.lkjmc.bindings;

public enum SyncDomain {
    CLAIMS(ClaimSnapshot.class),
    MENUS(MenuSnapshot.class),
    PERMISSIONS(PermissionSnapshot.class),
    PRESENCE(PresenceSnapshot.class),
    PROFILES(ProfileSnapshot.class),
    ROUTING(RoutingSnapshot.class),
    SETTINGS(SettingsSnapshot.class);
    private final Class<?> payloadType;
    SyncDomain(Class<?> type) { payloadType = type; }
    public Class<?> payloadType() { return payloadType; }
}
