package com.lkjmc.common.permission;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.time.Instant;
import java.util.Set;
import org.junit.jupiter.api.Test;

final class PermissionResolverTest {
    private static final Instant NOW = Instant.parse("2026-06-30T00:00:00Z");
    private final PermissionResolver resolver = new PermissionResolver();
    private final PrincipalIdentity alex = new PrincipalIdentity("minecraft-player", "alex", "Alex");

    @Test
    void platformPermissionAllowsWithoutSnapshot() {
        assertTrue(resolver.resolve(PermissionNodes.ADMIN_STATUS, true, false, null, NOW).allowed());
    }

    @Test
    void durableGrantAllowsWhenFresh() {
        var snapshot = snapshot(PermissionNodes.ADMIN_INSTANCE_LIST, NOW.plusSeconds(30));
        assertTrue(resolver.resolve(PermissionNodes.ADMIN_INSTANCE_LIST, false, false, snapshot, NOW).allowed());
    }

    @Test
    void staleSnapshotDoesNotEnableGrant() {
        var snapshot = snapshot(PermissionNodes.ADMIN_INSTANCE_LIST, NOW.minusSeconds(1));
        assertFalse(resolver.resolve(PermissionNodes.ADMIN_INSTANCE_LIST, false, false, snapshot, NOW).allowed());
    }

    @Test
    void superAdminGrantAllowsSpecificPermission() {
        var snapshot = snapshot(PermissionNodes.ADMIN_ADMIN, NOW.plusSeconds(30));
        assertTrue(resolver.resolve(PermissionNodes.ADMIN_INSTANCE_DELETE, false, false, snapshot, NOW).allowed());
    }

    @Test
    void missingPrincipalSnapshotDenies() {
        assertFalse(resolver.resolve(PermissionNodes.ADMIN_STATUS, false, false, null, NOW).allowed());
    }

    private PermissionSnapshot snapshot(String permission, Instant validUntil) {
        return new PermissionSnapshot(alex, Set.of(permission), NOW, validUntil);
    }
}
