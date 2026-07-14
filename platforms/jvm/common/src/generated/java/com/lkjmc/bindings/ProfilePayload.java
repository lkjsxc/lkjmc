package com.lkjmc.bindings;

public sealed interface ProfilePayload extends DomainPayload permits ProfileAvailable, ProfileMissing {}
