package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.Test;

final class ConfirmationPolicyTest {
    @Test
    void everyConfirmationRouteHasPolicyReason() {
        for (var id : List.of(
            "adventures-end-confirm", "adventures-end-party-confirm", "admin-server-stop-confirm",
            "admin-server-restart-confirm", "admin-server-delete-confirm", "admin-server-create-confirm",
            "claim-confirm", "claim-create-confirm", "party-confirm", "report-confirm",
            "home-update-confirm", "home-delete-confirm", "random-teleport-nether-confirm",
            "random-teleport-end-confirm")) {
            assertTrue(ConfirmationPolicy.reason(new MenuId(id)).isPresent(), id);
        }
    }

    @Test
    void safeRoutesDoNotRequireConfirmation() {
        for (var id : List.of("root", "settings", "language", "homes", "home-detail",
            "random-teleport-overworld", "shop", "achievements", "achievement-directory",
            "achievement-detail")) {
            assertFalse(ConfirmationPolicy.requiresConfirmation(new MenuId(id)), id);
        }
    }
}
