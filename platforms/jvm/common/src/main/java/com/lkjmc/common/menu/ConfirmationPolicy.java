package com.lkjmc.common.menu;

import java.util.Map;
import java.util.Optional;

public final class ConfirmationPolicy {
    private static final Map<String, String> REASONS = Map.ofEntries(
        Map.entry("adventures-end-confirm", "starts-temporary-infrastructure"),
        Map.entry("adventures-end-party-confirm", "starts-temporary-infrastructure"),
        Map.entry("admin-server-stop-confirm", "stops-server"),
        Map.entry("admin-server-restart-confirm", "forceful-server-mutation"),
        Map.entry("admin-server-delete-confirm", "deletes-durable-state"),
        Map.entry("admin-server-create-confirm", "starts-durable-resources"),
        Map.entry("claim-confirm", "deletes-durable-state"),
        Map.entry("claim-create-confirm", "creates-durable-world-state"),
        Map.entry("party-confirm", "affects-other-players"),
        Map.entry("report-confirm", "changes-moderation-state"),
        Map.entry("home-update-confirm", "overwrites-named-durable-state"),
        Map.entry("home-delete-confirm", "deletes-durable-state"),
        Map.entry("random-teleport-nether-confirm", "paid-dimension-change"),
        Map.entry("random-teleport-end-confirm", "paid-dimension-change")
    );

    private ConfirmationPolicy() {}

    public static Optional<String> reason(MenuId id) {
        return Optional.ofNullable(REASONS.get(id.value()));
    }

    public static boolean requiresConfirmation(MenuId id) {
        return reason(id).isPresent();
    }
}
