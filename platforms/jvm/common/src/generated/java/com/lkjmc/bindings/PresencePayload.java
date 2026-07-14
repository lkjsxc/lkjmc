package com.lkjmc.bindings;

public sealed interface PresencePayload extends DomainPayload permits PresenceAvailable, PresenceMissing {}
