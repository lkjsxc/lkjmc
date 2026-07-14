package com.lkjmc.bindings;

public sealed interface DomainPayload permits PermissionPayload, ClaimPayload, MenuPayload, RoutingPayload, SettingsPayload, ProfilePayload, PresencePayload {}
