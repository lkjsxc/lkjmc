package com.lkjmc.bindings;

public sealed interface DomainPayload permits PermissionPayload, ClaimPayload, RoutingPayload, SettingsPayload, ProfilePayload, PresencePayload {}
