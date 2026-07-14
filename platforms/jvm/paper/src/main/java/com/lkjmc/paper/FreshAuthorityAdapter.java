package com.lkjmc.paper;

import com.lkjmc.bindings.ClaimSnapshot;
import com.lkjmc.bindings.PermissionSnapshot;
import java.time.Instant;

public final class FreshAuthorityAdapter {
    public boolean permits(
            PermissionSnapshot snapshot,
            String principal,
            String permission,
            long requiredRevision,
            Instant now) {
        return snapshot != null && principal != null && permission != null && now != null
                && snapshot.principal().equals(principal)
                && snapshot.revision() == requiredRevision
                && snapshot.expiresAt().isAfter(now)
                && snapshot.permissions().contains(permission);
    }

    public boolean owns(
            ClaimSnapshot snapshot,
            String world,
            int chunkX,
            int chunkZ,
            java.util.UUID player,
            long requiredRevision,
            Instant now) {
        return snapshot != null && world != null && player != null && now != null
                && snapshot.revision() == requiredRevision
                && snapshot.expiresAt().isAfter(now)
                && snapshot.chunks().stream().anyMatch(chunk -> chunk.world().equals(world)
                    && chunk.chunkX() == chunkX && chunk.chunkZ() == chunkZ
                    && chunk.ownerUuid().equals(player));
    }
}
