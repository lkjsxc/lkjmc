package com.lkjmc.common.permission;

public record PermissionDecision(boolean allowed, String reason) {
    public PermissionDecision {
        reason = reason == null || reason.isBlank() ? "unknown" : reason;
    }

    public static PermissionDecision allow(String reason) {
        return new PermissionDecision(true, reason);
    }

    public static PermissionDecision deny(String reason) {
        return new PermissionDecision(false, reason);
    }
}
