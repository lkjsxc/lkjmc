package com.lkjmc.paper;

import com.lkjmc.bindings.ClaimSnapshot;
import com.lkjmc.bindings.PermissionSnapshot;
import java.time.Instant;
import java.util.UUID;

public final class FreshAuthorityAdapter {
    public boolean permits(
            PermissionSnapshot snapshot,
            String principal,
            String permission,
            long requiredRevision,
            Instant now) {
        return snapshot != null && principal != null && permission != null && now != null
                && "permissions".equals(snapshot.domain()) && snapshot.key().equals(principal)
                && snapshot.revision() == requiredRevision && !snapshot.generatedAt().isAfter(now)
                && (snapshot.payload().principalKind() + ":" + snapshot.payload().principalId())
                    .equals(principal)
                && snapshot.payload().permissions().contains(permission);
    }

    public boolean owns(
            ClaimSnapshot snapshot,
            String world,
            int chunkX,
            int chunkZ,
            UUID player,
            long requiredRevision,
            Instant now) {
        return snapshot != null && world != null && player != null && now != null
                && "claims".equals(snapshot.domain()) && snapshot.revision() == requiredRevision
                && !snapshot.generatedAt().isAfter(now)
                && snapshot.payload().chunks().stream().anyMatch(chunk -> chunk.worldName().equals(world)
                    && chunk.chunkX() == chunkX && chunk.chunkZ() == chunkZ
                    && chunk.ownerUuid().equals(player));
    }
}
