package com.lkjmc.common.menu;

public record ServerMenuEntry(
    String id,
    String kind,
    String desiredState,
    String observedState,
    boolean healthy,
    Integer playerCount,
    String connectHost,
    Integer connectPort,
    boolean proxyRegistrationDesired,
    boolean proxyRegistered,
    boolean joinable,
    String joinDisabledReason
) {
    public ServerMenuEntry(String id, String kind, String desiredState, String observedState,
                           boolean healthy, Integer playerCount) {
        this(id, kind, desiredState, observedState, healthy, playerCount, "", null, true, false, false,
            "menu.disabled.server-registration-unknown");
    }

    public ServerMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("server id is required");
        }
        kind = kind == null ? "unknown" : kind;
        desiredState = desiredState == null ? "unknown" : desiredState;
        observedState = observedState == null ? "unknown" : observedState;
        connectHost = connectHost == null ? "" : connectHost;
        joinDisabledReason = joinDisabledReason == null ? "" : joinDisabledReason;
    }
}
