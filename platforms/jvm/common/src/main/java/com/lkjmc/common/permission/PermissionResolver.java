package com.lkjmc.common.permission;

import java.time.Instant;

public final class PermissionResolver {
    public PermissionDecision resolve(
        String permission,
        boolean platformPermission,
        boolean operator,
        PermissionSnapshot snapshot,
        Instant now
    ) {
        if (permission == null || permission.isBlank()) {
            return PermissionDecision.deny("permission-missing");
        }
        if (platformPermission) {
            return PermissionDecision.allow("platform-permission");
        }
        if (operator) {
            return PermissionDecision.allow("operator");
        }
        if (snapshot == null) {
            return PermissionDecision.deny("snapshot-missing");
        }
        if (!snapshot.fresh(now)) {
            return PermissionDecision.deny("snapshot-stale");
        }
        return snapshot.grants(permission)
            ? PermissionDecision.allow("durable-grant")
            : PermissionDecision.deny("grant-missing");
    }
}
